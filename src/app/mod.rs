use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::TextArea;
use super::agent;
use crate::ui::*;

// 业务枚举，将原始按键翻译成在这个app中有意义的动作
pub enum Action {
    Quit,
    SwitchFocus,
    SendMessage,
    Up,
    Down,
}

pub struct App {
    // 程序退出
    pub should_quit: bool,

    // 智能体状态
    pub agent: agent::Agent,
}

// 挂载
impl App {
    pub fn new() -> Self {
        App {
            should_quit: false,
            agent: agent::Agent,
        }
    }

    // 按键触发入口
    pub fn handle_key(&mut self, key: KeyEvent, ui: &mut Ui) -> Option<Action> {
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
        match ui.focus {
            Focus::Input => self.handle_input_key(key, ui),
            Focus::SessionList => self.handle_session_key(key, ui),
        }
    }

    // 焦点在输入框的时候
    pub fn handle_input_key(&mut self, key: KeyEvent, ui: &mut Ui) -> Option<Action> {
        // 输入框的按键捕获
        // 同时匹配 Ctrl+Enter (某些终端) 和 Ctrl+j (Ctrl+Enter 的常见别名)
        let is_ctrl_enter = (key.code == KeyCode::Enter || key.code == KeyCode::Char('j'))
            && key.modifiers.contains(KeyModifiers::CONTROL);

        if is_ctrl_enter {
            return Some(Action::SendMessage);
        };

        ui.textarea.input(key);
        None
    }

    // 焦点在会话的时候
    pub fn handle_session_key(&mut self, key: KeyEvent, ui: &mut Ui) -> Option<Action> {
        // 在会话的按键捕获
        match key.code {
            // enter - 回到输入框
            KeyCode::Enter => {
                ui.focus = Focus::Input;
                None
            },
            // 向上滚动
            KeyCode::Up => Some(Action::Up),
            // 向下滚动
            KeyCode::Down => Some(Action::Down),

            // 没有封装的按键
            _ => None
        }
    }

    // 执行预设的业务，切换app状态
    pub async fn apply_action(&mut self, action: Action, ui: &mut Ui) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::SwitchFocus => {
                match ui.focus {
                    Focus::Input => ui.focus = Focus::SessionList,
                    Focus::SessionList => ui.focus = Focus::Input,
                }
            },
            Action::SendMessage => {
                let text = ui.textarea.lines().join("\n");
                if text.trim().is_empty() {
                    return;
                }

                // user消息：start → append → finish
                ui.session.start_user();
                ui.session.append_to_last(&text);
                ui.session.finish();

                // assistant消息：start → append → finish
                match self.agent.run(&ui.session.messages).await {
                    Ok(stream) => {
                        ui.stream = Some(stream);
                        ui.session.start_assistant();
                    }
                    Err(e) => {
                        ui.session.start_assistant();
                        ui.session.append_to_last(&format!("请求失败: {}", e));
                        ui.session.finish();
                    }
                }

                ui.textarea = TextArea::new(vec![]);
                ui.textarea.set_placeholder_text("在这里开始...");
            },
            Action::Up => {
                ui.scroll_offset = ui.scroll_offset.saturating_sub(1)
            },
            Action::Down => {
                ui.scroll_offset += 1
            },
        }
    }
}
