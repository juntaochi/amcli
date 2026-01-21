// src/ui/mod.rs
use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::player::{apple_music::AppleMusicController, MediaPlayer, RepeatMode, Track};

pub struct App {
    player: Box<dyn MediaPlayer>,
    current_track: Option<Track>,
    volume: u8,
    saved_volume: u8,
    is_muted: bool,
    show_help: bool,
    current_repeat_mode: RepeatMode,
}

impl App {
    pub async fn new() -> Result<Self> {
        let player = Box::new(AppleMusicController::new());
        let volume = player.get_volume().await.unwrap_or(50);

        Ok(Self {
            player,
            current_track: None,
            volume,
            saved_volume: volume,
            is_muted: false,
            show_help: false,
            current_repeat_mode: RepeatMode::Off,
        })
    }

    pub async fn toggle_playback(&mut self) -> Result<()> {
        self.player.toggle().await
    }

    pub async fn next_track(&mut self) -> Result<()> {
        self.player.next().await
    }

    pub async fn previous_track(&mut self) -> Result<()> {
        self.player.previous().await
    }

    pub async fn volume_up(&mut self) -> Result<()> {
        self.volume = (self.volume + 5).min(100);
        self.player.set_volume(self.volume).await?;
        self.is_muted = false;
        Ok(())
    }

    pub async fn volume_down(&mut self) -> Result<()> {
        self.volume = self.volume.saturating_sub(5);
        self.player.set_volume(self.volume).await?;
        self.is_muted = false;
        Ok(())
    }

    pub async fn toggle_mute(&mut self) -> Result<()> {
        if self.is_muted {
            self.volume = self.saved_volume;
            self.is_muted = false;
        } else {
            self.saved_volume = self.volume;
            self.volume = 0;
            self.is_muted = true;
        }
        self.player.set_volume(self.volume).await?;
        Ok(())
    }

    pub async fn seek_forward(&mut self) -> Result<()> {
        self.player.seek(5).await
    }

    pub async fn seek_backward(&mut self) -> Result<()> {
        self.player.seek(-5).await
    }

    pub fn navigate_up(&mut self) {}

    pub fn navigate_down(&mut self) {}

    pub fn navigate_left(&mut self) {}

    pub fn navigate_right(&mut self) {}

    pub async fn toggle_shuffle(&mut self) -> Result<()> {
        self.player.set_shuffle(true).await
    }

    pub async fn cycle_repeat(&mut self) -> Result<()> {
        self.current_repeat_mode = match self.current_repeat_mode {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
        self.player.set_repeat(self.current_repeat_mode).await
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    #[allow(dead_code)]
    pub fn is_showing_help(&self) -> bool {
        self.show_help
    }

    pub fn get_volume(&self) -> u8 {
        self.volume
    }

    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    #[allow(dead_code)]
    pub fn get_repeat_mode(&self) -> RepeatMode {
        self.current_repeat_mode
    }

    pub fn get_current_track(&self) -> Option<&Track> {
        self.current_track.as_ref()
    }

    pub async fn update(&mut self) -> Result<()> {
        self.current_track = self.player.get_current_track().await?;
        self.volume = self.player.get_volume().await.unwrap_or(self.volume);
        Ok(())
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("AMCLI - Apple Music Controller")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let content = if let Some(track) = app.get_current_track() {
        format!(
            "🎵 Now Playing\n\n\
             Track:    {}\n\
             Artist:   {}\n\
             Album:    {}\n\
             Duration: {}",
            track.name,
            track.artist,
            track.album,
            format_duration(track.duration)
        )
    } else {
        "No track playing\n\n\
         Press Space to start playback in Apple Music"
            .to_string()
    };

    let main_content = Paragraph::new(content).style(Style::default()).block(
        Block::default()
            .title("♫ Music Player")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    f.render_widget(main_content, chunks[1]);

    if let Some(track) = app.get_current_track() {
        let progress_percent = if track.duration.as_secs() > 0 {
            ((track.position.as_secs_f64() / track.duration.as_secs_f64()) * 100.0) as u16
        } else {
            0
        };

        let progress_label = format!(
            "{} / {}",
            format_duration(track.position),
            format_duration(track.duration)
        );

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .percent(progress_percent.min(100))
            .label(progress_label);

        f.render_widget(gauge, chunks[2]);
    } else {
        let empty_block = Block::default().borders(Borders::ALL).title("Progress");
        f.render_widget(empty_block, chunks[2]);
    }

    let help_text = format!(
        "[Space] Play/Pause | [[] Prev | []] Next | [+/-] Volume | [m] Mute | [s] Shuffle | [r] Repeat | [q] Quit  🔊 {}%{}",
        app.get_volume(),
        if app.is_muted() { " [MUTED]" } else { "" }
    );

    let status = Paragraph::new(help_text)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[3]);
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
