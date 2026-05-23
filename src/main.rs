
mod app;
mod ui;
mod session;
mod llm;
mod agent;

use crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;

#[tokio::main]
async  fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let mut app = app::App::new();

    // 主循环
    let result = run(&mut terminal, &mut app);

    // 无论结果如何，必须恢复终端，否则用户屏幕会乱
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>, app: &mut app::App, ) -> Result<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // 只处理 Press，忽略 Repeat/Release
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = app.handle_key(key) {
                        app.apply_action(action);
                    }
                }
            }
        }
    }

    Ok(())
}