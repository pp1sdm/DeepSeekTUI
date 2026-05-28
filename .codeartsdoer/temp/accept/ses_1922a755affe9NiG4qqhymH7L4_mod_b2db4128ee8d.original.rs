use super::session::Session;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::TextArea;
use super::agent;
use tokio::sync::mpsc;
use anyhow::Error;

pub enum ApiResult {
    Success(String),
    Error(String),
}

pub enum Focus {
    SessionList,
    Input,
}

pub enum Action {
    Quit,
    SwitchFocus,
    SendMessage,
}

pub struct App {
    pub focus: Focus,
    pub textarea: TextArea<'static>,
    pub should_quit: bool,
    pub is_loading: bool,

    pub session: Session,
    sender: Option<mpsc::UnboundedSender<ApiResult>>,
}

impl App {
    pub fn new() -> Self {
        let mut textarea = TextArea::new(vec![]);
        textarea.set_placeholder_text("在这里开始...");

        App {
            focus: Focus::Input,
            textarea,
            should_quit: false,
            is_loading: false,
            session: Session::new(),
            sender: None,
        }
    }

    pub fn set_sender(&mut self, sender: mpsc::UnboundedSender<ApiResult>) {
        self.sender = Some(sender);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(Action::Quit);
            }
            KeyCode::Tab => return Some(Action::SwitchFocus),
            _ => {}
        };

        match self.focus {
            Focus::Input => self.handle_input_key(key),
            Focus::SessionList => self.handle_session_key(key),
        }
    }

    pub fn handle_input_key(&mut self, key: KeyEvent) -> Option<Action> {
        if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL) {
            if self.is_loading {
                return None;
            }
            return Some(Action::SendMessage);
        };

        self.textarea.input(key);
        None
    }

    pub fn handle_session_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Enter => {
                self.focus = Focus::Input;
                None
            }
            _ => None,
        }
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::SwitchFocus => {
                match self.focus {
                    Focus::Input => self.focus = Focus::SessionList,
                    Focus::SessionList => self.focus = Focus::Input,
                }
            }
            Action::SendMessage => {
                let text = self.textarea.lines().join("\n");
                if text.trim().is_empty() {
                    return;
                }

                self.session.add_user_message(text);
                self.is_loading = true;
                self.textarea = TextArea::new(vec![]);

                if let Some(sender) = &self.sender {
                    let messages = self.session.messages.clone();
                    let sender = sender.clone();
                    tokio::spawn(async move {
                        let result = match agent::Agent.run(&messages).await {
                            Ok(reply) => ApiResult::Success(reply),
                            Err(e) => ApiResult::Error(format_error(&e)),
                        };
                        let _ = sender.send(result);
                    });
                } else {
                    self.is_loading = false;
                    self.session.add_assistant_message("[错误] 内部通道未初始化".to_string());
                }
            }
        }
    }

    pub fn handle_api_result(&mut self, result: ApiResult) {
        self.is_loading = false;
        match result {
            ApiResult::Success(reply) => {
                self.session.add_assistant_message(reply);
            }
            ApiResult::Error(msg) => {
                self.session.add_assistant_message(format!("[错误] {}", msg));
            }
        }
    }
}

fn format_error(e: &Error) -> String {
    let msg = e.to_string();
    if msg.contains("connection") || msg.contains("network") || msg.contains("dns") {
        format!("网络连接失败: {}", msg)
    } else if msg.contains("timeout") || msg.contains("timed out") {
        "请求超时，请检查网络连接后重试".to_string()
    } else if msg.contains("DEEPSEEK_KEY") {
        "API 密钥未配置，请在 .env 文件中设置 DEEPSEEK_KEY".to_string()
    } else if msg.contains("401") || msg.contains("Unauthorized") {
        "API 密钥无效，请检查 DEEPSEEK_KEY 是否正确".to_string()
    } else if msg.contains("429") {
        "请求过于频繁，请稍后重试".to_string()
    } else if msg.contains("500") || msg.contains("502") || msg.contains("503") {
        "DeepSeek 服务暂时不可用，请稍后重试".to_string()
    } else {
        msg
    }
}
