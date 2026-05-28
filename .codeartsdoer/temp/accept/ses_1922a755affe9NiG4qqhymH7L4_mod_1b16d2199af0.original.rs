use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::app::{App, Focus};

pub fn draw(f: &mut Frame, app: &mut App) {
    let main_area = f.area();

    let vertical_layout =
        Layout::vertical([Constraint::Min(1), Constraint::Length(6)]).split(main_area);

    render_session_list(f, app, vertical_layout[0]);
    render_input(f, app, vertical_layout[1]);
}

fn render_session_list(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = matches!(app.focus, Focus::SessionList);

    let block = Block::new()
        .title("会话列表")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let mut items: Vec<ListItem> = app
        .session
        .messages
        .iter()
        .map(|m| {
            let (prefix, style) = match m.role.as_str() {
                "user" => ("👤 ", Style::new()),
                "assistant" => {
                    if m.content.starts_with("[错误]") {
                        ("🤖 ", Style::new().fg(Color::Red))
                    } else {
                        ("🤖 ", Style::new())
                    }
                }
                _ => ("", Style::new()),
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(m.content.clone(), style),
            ]))
        })
        .collect();

    if app.is_loading {
        items.push(ListItem::new(
            Line::from(Span::styled("🤖 思考中...", Style::new().fg(Color::Cyan)))
        ));
    }

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

fn render_input(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = matches!(app.focus, Focus::Input);

    let title = if app.is_loading {
        " 输入框 (等待回复中...) "
    } else {
        " 输入框 "
    };

    let border_color = if app.is_loading {
        Color::DarkGray
    } else if is_focused {
        Color::Green
    } else {
        Color::DarkGray
    };

    let block = Block::new()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    f.render_widget(&app.textarea, inner);
}
