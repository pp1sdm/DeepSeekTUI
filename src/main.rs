
mod app;
mod ui;
mod session;
mod llm;
mod agent;
mod debug;
mod music;
mod display;
mod input;

use crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // debug部分
    let _log_guard = debug::init_log();

    // 创建终端
    let mut terminal = ratatui::init();

    // 创建应用实例
    let mut app = app::App::new();

    let result = run(&mut terminal, &mut app).await;

    ratatui::restore();

    result
}

async fn run<B>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut app::App,
) -> Result<(), Box<dyn std::error::Error>>
where
    B: ratatui::backend::Backend,
    B::Error: 'static,
{
    while !app.should_quit {
        // 绘制ui帧
        terminal.draw(|frame| ui::draw(frame, app))?;

        // 处理按键 - 阻塞线程，等待按键输入的同时还是动画帧绘制
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = app.handle_key(key) {
                        app.apply_action(action).await;
                    }
                }
            }
        }

        // 每轮都尝试拉取流数据
        app.poll_stream().await;

        // 每轮更新示波器数据
        app.update_scope_data();
    }

    Ok(())
}
