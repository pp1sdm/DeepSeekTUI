use super::session::Session;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::TextArea;
use super::agent;
use futures::{StreamExt, stream::BoxStream};
use super::display::{oscilloscope::Oscilloscope, spectroscope::Spectroscope, vectorscope::Vectorscope, GraphConfig};
use super::input::Matrix;

// 当前焦点区域
pub enum Focus {
    SessionList,
    Input,
    Scope,
}
// 业务枚举，将原始按键翻译成在这个app中有意义的动作
pub enum Action {
    Quit,
    SwitchFocus,
    SendMessage,
}

// 示波器状态枚举
pub enum CurrentDisplayMode {
    Oscilloscope,
    Vectorscope,
    Spectroscope,
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

    // 示波器状态
    pub scope_mode: CurrentDisplayMode,
    pub graph_config: GraphConfig,
    pub oscilloscope: Oscilloscope,
    pub spectroscope: Spectroscope,
    pub vectorscope: Vectorscope,
    pub scope_paused: bool,
    pub scope_data: Matrix<f64>,
    pub scope_tick: u64,
}

// 挂载
impl App {
    pub fn new() -> Self {
        let mut textarea = TextArea::new(vec![]);
        textarea.set_placeholder_text("在这里开始...");

        let graph_config = GraphConfig {
            pause: false,
            samples: 2048,
            sampling_rate: 48000,
            scale: 1.0,
            width: 2048,
            scatter: false,
            references: true,
            show_ui: true,
            marker_type: ratatui::symbols::Marker::Braille,
            palette: vec![
                ratatui::style::Color::Green,
                ratatui::style::Color::Yellow,
                ratatui::style::Color::Cyan,
                ratatui::style::Color::Magenta,
                ratatui::style::Color::Red,
            ],
            labels_color: ratatui::style::Color::White,
            axis_color: ratatui::style::Color::DarkGray,
        };

        let spectroscope = Spectroscope {
            sampling_rate: 48000,
            buffer_size: 2048,
            average: 1,
            buf: Vec::new(),
            window: false,
            log_y: true,
            phase_diff: false,
        };

        App {
            focus: Focus::Input,
            textarea,
            should_quit: false,
            session: Session::new(),
            agent: agent::Agent,
            stream: None,
            scope_mode: CurrentDisplayMode::Oscilloscope,
            graph_config,
            oscilloscope: Oscilloscope::default(),
            spectroscope,
            vectorscope: Vectorscope::default(),
            scope_paused: false,
            scope_data: vec![vec![]; 2],
            scope_tick: 0,
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
            _ => None,
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

    pub fn update_scope_data(&mut self) {
        if self.scope_paused {
            return;
        }
        self.scope_tick += 1;

        let all_text: String = self.session.messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("");

        let input_text: String = self.textarea.lines().join("");
        let combined = format!("{}{}", all_text, input_text);

        let samples = self.graph_config.samples as usize;
        let tick = self.scope_tick;

        if combined.is_empty() {
            let phase = tick as f64 * 0.1;
            let ch0: Vec<f64> = (0..samples)
                .map(|i| (i as f64 * 0.05 + phase).sin() * 0.3)
                .collect();
            let ch1: Vec<f64> = (0..samples)
                .map(|i| (i as f64 * 0.03 + phase * 1.3).sin() * 0.2)
                .collect();
            self.scope_data = vec![ch0, ch1];
            return;
        }

        let bytes = combined.as_bytes();
        let ch0: Vec<f64> = (0..samples)
            .map(|i| {
                let byte_idx = (i + tick as usize) % bytes.len();
                let raw = bytes[byte_idx] as f64 / 128.0 - 1.0;
                let noise = (i as f64 * 0.1 + tick as f64 * 0.07).sin() * 0.05;
                raw + noise
            })
            .collect();

        let ch1: Vec<f64> = (0..samples)
            .map(|i| {
                let byte_idx = (i + tick as usize + bytes.len() / 2) % bytes.len();
                let raw = bytes[byte_idx] as f64 / 128.0 - 1.0;
                let noise = (i as f64 * 0.08 + tick as f64 * 0.05).cos() * 0.05;
                raw + noise
            })
            .collect();

        self.scope_data = vec![ch0, ch1];
    }

    // 执行预设的业务，切换app状态
    pub async fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::SwitchFocus => {
                match self.focus {
                    Focus::Input => self.focus = Focus::SessionList,
                    Focus::SessionList => self.focus = Focus::Input,
                    Focus::Scope => self.focus = Focus::Input
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
