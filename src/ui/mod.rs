use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::app::{App, Focus, CurrentDisplayMode};
use crate::display::{Dimension, DisplayMode};

// 颜色变量
const COLOR_PRIMARY: Color = Color::Rgb(0, 255, 168);
const COLOR_DIM: Color = Color::Rgb(80, 80, 80);
const COLOR_USER_MSG: Color = Color::Rgb(137, 212, 255);
const COLOR_ASSISTANT_MSG: Color = Color::Rgb(168, 230, 160);
const COLOR_BG: Color = Color::Rgb(0, 0, 0);
const COLOR_THINKING: Color = Color::Rgb(255, 200, 50);
const COLOR_READY: Color = Color::Rgb(0, 255, 100);
const COLOR_BORDER_UNFOCUS: Color = Color::Rgb(60, 60, 60);

// 唯一入口，根据app的状态绘制
pub fn draw(f: &mut Frame, app: &mut App) {
    // 框架的区域
    let main_area = f.area();

    // 分割四个区域
    let vertical_layout =
        Layout::vertical([
            Constraint::Length(1),   // 状态栏
            Constraint::Min(1),      // 会话
            Constraint::Length(1),   // 分隔线
            Constraint::Length(6),   // 输入框
        ]).split(main_area);

    // 将分割渲染给到各个区域
    render_status_bar(f, app, vertical_layout[0]);
    // 根据焦点切换中间区域
    // match app.focus {
    //     Focus::Scope => ,
    //     _ => render_session_list(f, app, vertical_layout[1]),
    // }
    render_scope(f, app, vertical_layout[1]);
    render_separator(f, vertical_layout[2]);
    render_input(f, app, vertical_layout[3]);
}

// 渲染会话列表
fn render_session_list(f: &mut Frame, app: &mut App, area: Rect) {
    // 焦点集中
    let is_focused = matches!(app.focus, Focus::SessionList);
    let border_color = if is_focused { COLOR_PRIMARY } else { COLOR_BORDER_UNFOCUS };

    // 当前块
    let block = Block::new()
        .title(Line::from(vec![
            Span::styled(" 💬 会话 ", Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(border_color));  // 关键：边框着色

    let items: Vec<ListItem> = app
        .session
        .messages
        .iter()
        .map(|m| {
            let (icon, color) = match m.role.as_str() {
                "user" => (" 👤 ", COLOR_USER_MSG),
                "assistant" => (" 🤖 ", COLOR_ASSISTANT_MSG),
                _ => ("   ", Color::White),
            };

            ListItem::new(Line::from(vec![
                Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(&m.content, Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

// 渲染示波器
fn render_scope(f: &mut Frame, app: &mut App, area: Rect) {
    let mut all_datasets = Vec::new();
    let cfg = app.graph_config.clone();
    let data = &app.scope_data;

    match app.scope_mode {
        CurrentDisplayMode::Oscilloscope => {
            if cfg.references {
                all_datasets.extend(app.oscilloscope.references(&cfg));
            }
            all_datasets.extend(app.oscilloscope.process(&cfg, data));
        }
        CurrentDisplayMode::Vectorscope => {
            if cfg.references {
                all_datasets.extend(app.vectorscope.references(&cfg));
            }
            all_datasets.extend(app.vectorscope.process(&cfg, data));
        }
        CurrentDisplayMode::Spectroscope => {
            if cfg.references {
                all_datasets.extend(app.spectroscope.references(&cfg));
            }
            all_datasets.extend(app.spectroscope.process(&cfg, data));
        }
    }

    let x_axis = match app.scope_mode {
        CurrentDisplayMode::Oscilloscope => app.oscilloscope.axis(&cfg, Dimension::X),
        CurrentDisplayMode::Vectorscope => app.vectorscope.axis(&cfg, Dimension::X),
        CurrentDisplayMode::Spectroscope => app.spectroscope.axis(&cfg, Dimension::X),
    };
    let y_axis = match app.scope_mode {
        CurrentDisplayMode::Oscilloscope => app.oscilloscope.axis(&cfg, Dimension::Y),
        CurrentDisplayMode::Vectorscope => app.vectorscope.axis(&cfg, Dimension::Y),
        CurrentDisplayMode::Spectroscope => app.spectroscope.axis(&cfg, Dimension::Y),
    };

    let chart = Chart::new(all_datasets.iter().map(|ds| ds.into()).collect::<Vec<_>>())
        .x_axis(x_axis)
        .y_axis(y_axis);

    let mode_name = match app.scope_mode {
        CurrentDisplayMode::Oscilloscope => "示波器",
        CurrentDisplayMode::Spectroscope => "频谱仪",
        CurrentDisplayMode::Vectorscope => "矢量示波器",
    };

    let block = Block::new()
        .title(Line::from(vec![
            Span::styled(
                format!(" ◆ {}", mode_name),
                Style::default().fg(COLOR_PRIMARY).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.scope_paused { " ⏸ 暂停 " } else { " ▶ 运行 " },
                Style::default().fg(if app.scope_paused { COLOR_THINKING } else { COLOR_READY }),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_PRIMARY));

    f.render_widget(chart.block(block), area);
}

// 渲染输入框
fn render_input(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = matches!(app.focus, Focus::Input);
    let border_color = if is_focused { COLOR_PRIMARY } else { COLOR_BORDER_UNFOCUS };

    let block = Block::new()
        .title(Line::from(vec![
            Span::styled(" ✎ 输入 ", Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(&app.textarea, inner);
}

// 渲染状态栏
fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let is_streaming = app.stream.is_some();
    let focus_name = match app.focus {
        Focus::SessionList => "会话",
        Focus::Input => "输入",
        Focus::Scope => "示波器",
    };

    let spans = vec![
        // 品牌标题：青绿加粗
        Span::styled(" ◆ DeepSeek ", Style::default().fg(COLOR_PRIMARY).add_modifier(Modifier::BOLD)),
        // 状态指示：生成中用金色，就绪用绿色
        Span::styled(
            if is_streaming { " ● Thinking... " } else { " ○ Ready " },
            Style::default().fg(if is_streaming { COLOR_THINKING } else { COLOR_READY }),
        ),
        // 快捷键：暗灰色弱化
        Span::styled(
            format!(" Tab:切换 │ Ctrl+Enter:发送 │ 焦点:{}", focus_name),
            Style::default().fg(COLOR_DIM),
        ),
    ];

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(COLOR_BG)),
        area,
    );
}

// 渲染分割线
fn render_separator(f: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            line,
            Style::default().fg(COLOR_BORDER_UNFOCUS).bg(COLOR_BG),
        ))),
        area,
    );
}
