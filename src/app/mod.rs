use super::session::Session;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::{TextArea, WrapMode, Input};
use super::agent;

// 当前焦点区域
pub enum Focus {
    SessionList,
    Input,
}

// 业务枚举，将原始按键翻译成在这个app中有意义的动作
enum Action {
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
}

// 挂载
impl App {
    pub fn new() -> Self {
        let mut textarea = TextArea::new(vec![]);
        textarea.set_placeholder_text(
            "在这里开始..."
        );
        textarea.set_wrap_mode(
            WrapMode::WordOrGlyph
        );

        App {
            focus: Focus::Input,
            textarea,
            should_quit: false,
            session: Session::new(),
            agent: agent::Agent,
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
        // 在输入框的按键捕获
        // enter + ctrl 提交消息
        if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Some(Action::SendMessage);
        };

        // 其余按键给到textarea处理
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
                // 获取输入框的文本
                let text = self.textarea.lines().join("\n");

                // 将信息给到会话，用用户的身份
                self.session.add_user_message(text);

                // 调用智能体
                let res = self.agent.run(&self.session.messages.as_slice()).await.unwrap();

                // 将信息给到会话，用智能体的身份
                self.session.add_assistant_message(res);

                // 清空输入框
                self.textarea.clear();
            },
        }
    }
}
