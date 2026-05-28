
mod app;
mod ui;
mod session;
mod llm;
mod agent;
mod debug;

use crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;
use dotenvy::dotenv;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    debug::init_log();

    let mut terminal = ratatui::init();

    let (api_sender, mut api_receiver) = mpsc::unbounded_channel::<app::ApiResult>();

    let mut app = app::App::new();
    app.set_sender(api_sender);

    let result = run(&mut terminal, &mut app, &mut api_receiver).await;

    ratatui::restore();

    result
}

async fn run<B>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut app::App,
    api_receiver: &mut mpsc::UnboundedReceiver<app::ApiResult>,
) -> Result<(), Box<dyn std::error::Error>>
where
    B: ratatui::backend::Backend,
    B::Error: 'static,
{
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        while let Ok(result) = api_receiver.try_recv() {
            app.handle_api_result(result);
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
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
