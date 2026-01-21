// src/main.rs
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

mod artwork;
mod config;
mod lyrics;
mod player;
mod ui;

use crate::ui::App;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _args = Args::parse();
    tracing_subscriber::fmt::init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new().await?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()>
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit
                    KeyCode::Char('q') => return Ok(()),

                    // Playback control
                    KeyCode::Char(' ') => app.toggle_playback().await?,
                    KeyCode::Char(']') => app.next_track().await?,
                    KeyCode::Char('[') => app.previous_track().await?,

                    // Volume control
                    KeyCode::Char('=') | KeyCode::Char('+') => app.volume_up().await?,
                    KeyCode::Char('-') | KeyCode::Char('_') => app.volume_down().await?,
                    KeyCode::Char('m') => app.toggle_mute().await?,

                    // Seek control
                    KeyCode::Right => app.seek_forward().await?,
                    KeyCode::Left => app.seek_backward().await?,
                    KeyCode::Char('.') => app.seek_forward().await?,
                    KeyCode::Char(',') => app.seek_backward().await?,

                    // Navigation (for future views)
                    KeyCode::Char('k') | KeyCode::Up => app.navigate_up(),
                    KeyCode::Char('j') | KeyCode::Down => app.navigate_down(),
                    KeyCode::Char('h') => app.navigate_left(),
                    KeyCode::Char('l') => app.navigate_right(),

                    // Mode toggles
                    KeyCode::Char('s') => app.toggle_shuffle().await?,
                    KeyCode::Char('r') => app.cycle_repeat().await?,
                    KeyCode::Char('t') => app.next_theme().await?,

                    // Help
                    KeyCode::Char('?') => app.toggle_help(),

                    _ => {}
                }
            }
        }

        // Update app state
        app.update().await?;
    }
}
