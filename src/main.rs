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
    let mut last_update = std::time::Instant::now();
    let update_interval = std::time::Duration::from_millis(500);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    // Check for Ctrl+C first
                    if key.code == KeyCode::Char('c') 
                        && key.modifiers.contains(event::KeyModifiers::CONTROL) 
                    {
                        return Ok(());
                    }

                    // Handle settings menu navigation if open
                    if app.is_settings_open() {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('S') => {
                                app.close_settings();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.settings_navigate_up();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.settings_navigate_down();
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                app.settings_select().await?;
                            }
                            _ => {}
                        }
                    } else {
                        // Normal app controls when settings not open
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('s') | KeyCode::Char('S') => app.toggle_settings_menu(),
                            KeyCode::Char(' ') => app.toggle_playback().await?,
                            KeyCode::Char(']') => app.next_track().await?,
                            KeyCode::Char('[') => app.previous_track().await?,
                            KeyCode::Char('=') | KeyCode::Char('+') => app.volume_up().await?,
                            KeyCode::Char('-') | KeyCode::Char('_') => app.volume_down().await?,
                            KeyCode::Char('m') => app.toggle_mute().await?,
                            KeyCode::Right => app.seek_forward().await?,
                            KeyCode::Left => app.seek_backward().await?,
                            KeyCode::Char('.') => app.seek_forward().await?,
                            KeyCode::Char(',') => app.seek_backward().await?,
                            KeyCode::Char('k') | KeyCode::Up => app.navigate_up(),
                            KeyCode::Char('j') | KeyCode::Down => app.navigate_down(),
                            KeyCode::Char('h') => app.navigate_left(),
                            KeyCode::Char('l') => app.navigate_right(),
                            KeyCode::Char('r') => app.cycle_repeat().await?,
                            KeyCode::Char('t') => app.next_theme().await?,
                            KeyCode::Char('?') => app.toggle_help(),
                            _ => {}
                        }
                    }
                }
                Event::Mouse(_mouse) => {
                    // Mouse support placeholder - we'll implement detailed handling next
                    // For now, we just consume the event
                }
                _ => {}
            }
        }

        if last_update.elapsed() >= update_interval {
            app.update().await?;
            last_update = std::time::Instant::now();
        }
    }
}
