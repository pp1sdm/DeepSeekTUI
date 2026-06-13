
mod app;
mod ui;
mod session;
mod llm;
mod agent;
mod debug;
mod display;
mod input;
mod data;

use crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // debug部分
    let _log_guard = debug::init_log();

    // 链接数据库
    let db_url = "sqlite:data/memory.db";
    let pool = SqlitePool::connect(db_url).await?;
    // 初始化建表，拿到数据库实例
    let memory_db = data::MemoryDB::new(pool).await?;

    // 创建终端
    let mut terminal = ratatui::init();

    // 创建应用实例
    let mut app = app::App::new();

    // 创建ui实例
    let mut ui = ui::Ui::new();

    let result = run(&mut terminal, &mut app, &mut ui).await;

    ratatui::restore();

    result
}

async fn run<B>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut app::App,
    ui: &mut ui::Ui,
) -> Result<(), Box<dyn std::error::Error>>
where
    B: ratatui::backend::Backend,
    B::Error: 'static,
{
    while !app.should_quit {
        // 绘制ui帧
        terminal.draw(|frame| ui.draw(frame))?;

        // 处理按键 - 阻塞线程，等待按键输入的同时还是动画帧绘制
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = app.handle_key(key, ui) {
                        app.apply_action(action, ui).await;
                    }
                }
            }
        }

        // 每轮都尝试拉取流数据
        ui.poll_stream().await;

        // 每轮更新示波器数据
        ui.update_scope_data();
    }

    Ok(())
}
