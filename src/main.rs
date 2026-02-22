mod api;
mod app;
mod config;
mod event;
mod scaffold;
mod ui;

use anyhow::Result;
use std::time::Duration;

use app::App;
use config::Config;
use event::EventHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    let mut terminal = ratatui::init();
    let mut events = EventHandler::new(Duration::from_millis(100));
    let mut app = App::new(config)?;

    let result = app.run(&mut terminal, &mut events).await;

    ratatui::restore();

    // Print last opened directory so a shell wrapper can cd into it
    if let Some(dir) = &app.last_opened_dir {
        println!("{}", dir.display());
    }

    result
}
