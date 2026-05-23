use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Focus};

// 唯一入口，根据app的状态绘制
pub fn draw(f: &mut Frame, app: &mut App) {
    // 框架的区域
    let main_area = f.area();

    // 上下分割
    let vertical_layout =
        Layout::vertical([Constraint::Min(1), Constraint::Length(6)]).split(main_area);

    // 将分割给到两个区域
    render_session_list(f, app, vertical_layout[0]);
    render_input(f, app, vertical_layout[1]);
}

// 渲染会话列表
fn render_session_list(f: &mut Frame, app: &mut App, area: Rect) {
    // 判断当前的焦点位置
    let is_focused = matches!(app.focus, Focus::SessionList);

    // 当前的块
    let block = Block::new()
        .title("会话列表")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    // 渲染元素
    let items: Vec<ListItem> = app
        .session
        .messages
        .iter()
        .map(|m| ListItem::new(m.content.clone()))
        .collect();

    // 创建当前渲染列表
    let list = List::new(items).block(block);

    // 框架渲染组件
    f.render_widget(list, area);
}

// 渲染输入框
fn render_input(f: &mut Frame, app: &mut App, area: Rect) {
    // 判断当前的焦点位置
    let is_focused = matches!(app.focus, Focus::Input);

    // 当前块
    let block = Block::new()
        .title(" 输入框 ")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::new().fg(Color::Green)
        } else {
            Style::new().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    f.render_widget(&app.textarea, inner);
}
