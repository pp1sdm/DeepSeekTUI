use super::session::Session;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::TextArea;
use super::agent;
use futures::{StreamExt, stream::BoxStream};

// 当前焦点区域
pub enum Focus {
    SessionList,
    Input,
}

// 业务枚举，将原始按键翻译成在这个app中有意义的动作
pub enum Action {
    Quit,
    SwitchFocus,
    SendMessage,
}
pub struct App {
    // app的基本状态
    pub focus: Focus,
    pub textarea: TextArea<'static>,
    pub should_quit: bool,

    // app的业务状态
    pub session: Session,
    pub agent: agent::Agent,

    // 流式状态
    pub stream: Option<BoxStream<'static, anyhow::Result<String>>>,
}

// 挂载
impl App {
    pub fn new() -> Self {
        let mut textarea = TextArea::new(vec![]);
        textarea.set_placeholder_text(
            "在这里开始..."
        );

        App {
            focus: Focus::Input,
            textarea,
            should_quit: false,
            session: Session::new(),
            agent: agent::Agent,
            stream: None,
        }
    }

    // 按键触发入口
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        // 全局最高优先级的按键捕获
        match key.code {
            // ctrl + c 直接退出
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(Action::Quit);
            },

            // tab键切换焦点
            KeyCode::Tab => return Some(Action::SwitchFocus),

            // 没有封装的按键
            _ => {}
        };

        // 根据焦点区域处理按键
        match self.focus {
            Focus::Input => self.handle_input_key(key),
            Focus::SessionList => self.handle_session_key(key),
        }
    }

    // 焦点在输入框的时候
    pub fn handle_input_key(&mut self, key: KeyEvent) -> Option<Action> {
        // 输入框的按键捕获
        // 同时匹配 Ctrl+Enter (某些终端) 和 Ctrl+j (Ctrl+Enter 的常见别名)
        let is_ctrl_enter = (key.code == KeyCode::Enter || key.code == KeyCode::Char('j'))
            && key.modifiers.contains(KeyModifiers::CONTROL);

        if is_ctrl_enter {
            return Some(Action::SendMessage);
        };

        self.textarea.input(key);
        None
    }

    // 焦点在会话的时候
    pub fn handle_session_key(&mut self, key: KeyEvent) -> Option<Action> {
        // 在会话的按键捕获
        match key.code {
            // enter - 回到输入框
            KeyCode::Enter => {
                self.focus = Focus::Input;
                None
            },

            // 没有封装的按键
            _ => None
        }
    }

    // 流式添加数据
    pub async fn poll_stream(&mut self) {
        let stream = match &mut self.stream {
            Some(s) => s,
            None => return,
        };

        match stream.next().await {
            Some(Ok(chunk)) => {
                self.session.append_to_last(&chunk);
            }
            Some(Err(e)) => {
                self.session.append_to_last(&format!("\n[错误: {}]", e));
                self.stream = None;
                self.session.finish();
            }
            None => {
                self.stream = None;
                self.session.finish();
            }
        }
    }

    // 执行预设的业务，切换app状态
    pub async fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::SwitchFocus => {
                match self.focus {
                    Focus::Input => self.focus = Focus::SessionList,
                    Focus::SessionList => self.focus = Focus::Input,
                }
            },
            Action::SendMessage => {
                let text = self.textarea.lines().join("\n");
                if text.trim().is_empty() {
                    return;
                }

                // user消息：start → append → finish
                self.session.start_user();
                self.session.append_to_last(&text);
                self.session.finish();

                // assistant消息：start → append → finish
                match self.agent.run(&self.session.messages).await {
                    Ok(stream) => {
                        self.stream = Some(stream);
                        self.session.start_assistant();
                    }
                    Err(e) => {
                        self.session.start_assistant();
                        self.session.append_to_last(&format!("请求失败: {}", e));
                        self.session.finish();
                    }
                }

                self.textarea = TextArea::new(vec![]);
                self.textarea.set_placeholder_text("在这里开始...");
            }
        }
    }
}
