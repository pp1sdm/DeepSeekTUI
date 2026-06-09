use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, BorderType, Borders, Chart, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::display::{Dimension, DisplayMode};
use crate::ui::{Focus, Ui, COLOR_ASSISTANT_MSG, COLOR_BG, COLOR_BORDER_UNFOCUS, COLOR_DIM, COLOR_PRIMARY, COLOR_READY, COLOR_THINKING, COLOR_USER_MSG};

// 挂载渲染函数
impl Ui {
    // 唯一入口，根据app的状态绘制
    pub fn draw(&mut self, f: &mut Frame) {
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
        self.render_status_bar(f, vertical_layout[0]);
        self.render_scope(f, vertical_layout[1]);
        self.render_separator(f, vertical_layout[2]);
        self.render_input(f, vertical_layout[3]);
    }

    // 渲染状态栏
    pub fn render_status_bar(&mut self, f: &mut Frame, area: Rect) {
        let is_streaming = self.stream.is_some();
        let focus_name = match self.focus {
            Focus::SessionList => "会话",
            Focus::Input => "输入",
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
    // 渲染示波器
    pub fn render_scope(&mut self, f: &mut Frame, area: Rect) {
        // ========== 示波器（背景，完全不变）==========
        let mut all_datasets = Vec::new();
        let cfg = self.graph_config.clone();
        let data = &self.scope_data;
        all_datasets.extend(self.oscilloscope.process(&cfg, data));

        let x_axis = self.oscilloscope.axis(&cfg, Dimension::X);
        let y_axis = self.oscilloscope.axis(&cfg, Dimension::Y);

        let chart = Chart::new(all_datasets.iter().map(|ds| ds.into()).collect::<Vec<_>>())
            .x_axis(x_axis)
            .y_axis(y_axis);

        let mode_name = "示波器";
        let chart_block = Block::new()
            .title(Line::from(vec![
                Span::styled(format!(" ◆ {}", mode_name), Style::default().fg(COLOR_PRIMARY).add_modifier(Modifier::BOLD)),
                Span::styled(
                    if self.scope_paused { " ⏸ 暂停 " } else { " ▶ 运行 " },
                    Style::default().fg(if self.scope_paused { COLOR_THINKING } else { COLOR_READY }),
                ),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_PRIMARY));

        f.render_widget(chart.block(chart_block), area);

        // ========== 会话（前景，带滚动条）==========
        let is_focused = matches!(self.focus, Focus::SessionList);
        let border_color = if is_focused { COLOR_PRIMARY } else { COLOR_BORDER_UNFOCUS };

        let block = Block::new()
            .title(Line::from(vec![
                Span::styled(" 💬 会话 ", Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        let max_width = inner.width as usize;

        // 手动按 Unicode 显示宽度预切分，得到真实行数
        let mut all_lines: Vec<Line> = Vec::new();
        for m in &self.session.messages {
            let (icon, color) = match m.role.as_str() {
                "user" => (" 👤 ", COLOR_USER_MSG),
                "assistant" => (" 🤖 ", COLOR_ASSISTANT_MSG),
                _ => ("   ", Color::White),
            };

            let icon_width = icon.width();
            let content_width = max_width.saturating_sub(icon_width);
            if content_width == 0 { continue; }

            let mut chunks: Vec<String> = Vec::new();
            let mut cur = String::new();
            let mut cur_w = 0;

            for ch in m.content.chars() {
                let w = ch.width().unwrap_or(0);
                if cur_w + w > content_width && !cur.is_empty() {
                    chunks.push(cur);
                    cur = String::new();
                    cur_w = 0;
                }
                cur.push(ch);
                cur_w += w;
            }
            if !cur.is_empty() { chunks.push(cur); }

            for (i, piece) in chunks.into_iter().enumerate() {
                let prefix = if i == 0 { icon } else { "    " };
                all_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(piece, Style::default().fg(color)),
                ]));
            }
        }

        let inner_height = inner.height as usize;
        let total_lines = all_lines.len();
        let max_scroll = total_lines.saturating_sub(inner_height);

        // 限制偏移不越界（但不要自动拉到底部，由用户控制）
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }

        let list = Paragraph::new(all_lines)
            .block(block)
            .scroll((self.scroll_offset as u16, 0));

        f.render_widget(list, area);

        // ========== 滚动条（只有内容超出时才显示）==========
        if total_lines > inner_height {
            let scrollbar_area = Rect::new(
                inner.x + inner.width.saturating_sub(1),
                inner.y,
                1,
                inner.height,
            );

            let mut scrollbar_state = ScrollbarState::new(total_lines)
                .position(self.scroll_offset)
                .viewport_content_length(inner_height);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .thumb_symbol("█");

            f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    // 渲染分割线
    pub fn render_separator(&mut self, f: &mut Frame, area: Rect) {
        let line = "─".repeat(area.width as usize);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                line,
                Style::default().fg(COLOR_BORDER_UNFOCUS).bg(COLOR_BG),
            ))),
            area,
        );
    }

    // 渲染输入框
    pub fn render_input(&mut self, f: &mut Frame, area: Rect) {
        let is_focused = matches!(self.focus, Focus::Input);
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
        f.render_widget(&self.textarea, inner);
    }
}