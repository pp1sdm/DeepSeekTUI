mod render;
mod update;

use futures::stream::BoxStream;
use futures::StreamExt;
use ratatui::{
    prelude::*,
    widgets::*,
};
use ratatui_textarea::TextArea;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::app::{App};
use crate::display::{Dimension, DisplayMode, GraphConfig};
use crate::display::oscilloscope::Oscilloscope;
use crate::input::Matrix;
use crate::session::Session;

// 颜色变量
const COLOR_PRIMARY: Color = Color::Rgb(0, 255, 168);
const COLOR_DIM: Color = Color::Rgb(80, 80, 80);
const COLOR_USER_MSG: Color = Color::Rgb(137, 212, 255);
const COLOR_ASSISTANT_MSG: Color = Color::Rgb(168, 230, 160);
const COLOR_BG: Color = Color::Rgb(0, 0, 0);
const COLOR_THINKING: Color = Color::Rgb(255, 200, 50);
const COLOR_READY: Color = Color::Rgb(0, 255, 100);
const COLOR_BORDER_UNFOCUS: Color = Color::Rgb(60, 60, 60);

pub enum Focus {
    SessionList,
    Input,
}
// ui构造
pub struct Ui {
    // ui渲染状态
    pub textarea: TextArea<'static>,
    pub session: Session,

    // 当前的焦点
    pub focus: Focus,

    // 流式状态
    pub stream: Option<BoxStream<'static, anyhow::Result<String>>>,

    // 示波器状态
    pub graph_config: GraphConfig,
    pub oscilloscope: Oscilloscope,
    pub scope_paused: bool,
    pub scope_data: Matrix<f64>,
    pub scope_tick: u64,

    // 滚动条
    pub scroll_offset: usize
}

impl Ui {
    pub fn new() -> Self {
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
        let mut textarea = TextArea::new(vec![]);
        textarea.set_placeholder_text("在这里开始...");
        Self {
            focus: Focus::Input,
            session: Session::new(),
            textarea,
            stream: None,
            graph_config,
            oscilloscope: Oscilloscope::default(),
            scope_paused: false,
            scope_data: vec![vec![]; 2],
            scope_tick: 0,
            scroll_offset: 0
        }
    }
}

