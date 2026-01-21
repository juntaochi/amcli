use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Wrap},
    Frame,
};
use std::time::Duration;

use crate::artwork::converter::ArtworkConverter;
use crate::artwork::ArtworkManager;
use crate::player::{apple_music::AppleMusicController, MediaPlayer, RepeatMode, Track};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::StatefulImage;
use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse, BRAILLE_SIX_DOUBLE};

pub const COLOR_BG: Color = Color::Rgb(0, 0, 0);
pub const COLOR_TEXT_DIM: Color = Color::Rgb(80, 60, 20);
pub const COLOR_TEXT_BRIGHT: Color = Color::Rgb(255, 176, 0);
pub const COLOR_ACCENT: Color = Color::Rgb(255, 215, 0);
pub const COLOR_ALERT: Color = Color::Rgb(255, 50, 50);

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub primary: Color,
    pub dim: Color,
    pub accent: Color,
    pub alert: Color,
}

pub const THEME_AMBER_RETRO: Theme = Theme {
    name: "AMBER VFD",
    primary: COLOR_TEXT_BRIGHT,
    dim: COLOR_TEXT_DIM,
    accent: COLOR_ACCENT,
    alert: COLOR_ALERT,
};

pub const THEME_GREEN_VFD: Theme = Theme {
    name: "GREEN VFD",
    primary: Color::Rgb(0, 255, 65),
    dim: Color::Rgb(0, 80, 20),
    accent: Color::Rgb(50, 255, 100),
    alert: Color::Rgb(255, 100, 0),
};

pub const THEME_CYAN_VFD: Theme = Theme {
    name: "CYAN VFD",
    primary: Color::Rgb(0, 255, 255),
    dim: Color::Rgb(0, 80, 100),
    accent: Color::Rgb(0, 150, 255),
    alert: Color::Rgb(255, 50, 50),
};

pub const THEME_RED_ALERT: Theme = Theme {
    name: "RED ALERT",
    primary: Color::Rgb(255, 50, 50),
    dim: Color::Rgb(100, 0, 0),
    accent: Color::Rgb(255, 100, 100),
    alert: Color::Rgb(255, 255, 0),
};

pub const THEMES: &[Theme] = &[
    THEME_AMBER_RETRO,
    THEME_GREEN_VFD,
    THEME_CYAN_VFD,
    THEME_RED_ALERT,
];

pub struct App {
    player: Box<dyn MediaPlayer>,
    current_track: Option<Track>,
    volume: u8,
    saved_volume: u8,
    is_muted: bool,
    show_help: bool,
    current_repeat_mode: RepeatMode,
    artwork_manager: ArtworkManager,
    artwork_converter: ArtworkConverter,
    artwork_protocol: Option<StatefulProtocol>,
    current_artwork_url: Option<String>,
    is_loading_artwork: bool,
    throbber_state: ThrobberState,
    current_theme_index: usize,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = crate::config::Config::load()?;
        let player = Box::new(AppleMusicController::new());
        Self::with_player_and_config(player, config).await
    }

    #[allow(dead_code)]
    pub async fn with_player(player: Box<dyn MediaPlayer>) -> Result<Self> {
        let config = crate::config::Config::load()?;
        Self::with_player_and_config(player, config).await
    }

    pub async fn with_player_and_config(player: Box<dyn MediaPlayer>, config: crate::config::Config) -> Result<Self> {
        let volume = player.get_volume().await.unwrap_or(50);
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("amcli/artwork");

        Ok(Self {
            player,
            current_track: None,
            volume,
            saved_volume: volume,
            is_muted: false,
            show_help: false,
            current_repeat_mode: RepeatMode::Off,
            artwork_manager: ArtworkManager::new(cache_dir),
            artwork_converter: ArtworkConverter::with_mode(&config.artwork.mode)?,
            artwork_protocol: None,
            current_artwork_url: None,
            is_loading_artwork: false,
            throbber_state: ThrobberState::default(),
            current_theme_index: 0,
        })
    }

    pub fn current_theme(&self) -> Theme {
        THEMES[self.current_theme_index]
    }

    pub async fn next_theme(&mut self) -> Result<()> {
        self.current_theme_index = (self.current_theme_index + 1) % THEMES.len();
        self.current_artwork_url = None;
        self.artwork_protocol = None;
        self.update().await?;
        Ok(())
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

    pub fn get_current_track(&self) -> Option<&Track> {
        self.current_track.as_ref()
    }

    #[allow(dead_code)]
    pub fn get_volume(&self) -> u8 {
        self.volume
    }

    #[allow(dead_code)]
    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    #[allow(dead_code)]
    pub fn get_repeat_mode(&self) -> RepeatMode {
        self.current_repeat_mode
    }

    pub async fn update(&mut self) -> Result<()> {
        self.current_track = self.player.get_current_track().await?;
        self.volume = self.player.get_volume().await.unwrap_or(self.volume);
        self.throbber_state.calc_next();

        let artwork_url = self.player.get_artwork_url().await.unwrap_or(None);
        if artwork_url != self.current_artwork_url {
            self.current_artwork_url = artwork_url.clone();
            if let Some(url) = artwork_url {
                self.is_loading_artwork = true;
                let theme = self.current_theme();
                if let Ok(img) = self
                    .artwork_manager
                    .get_artwork_themed(&url, theme.dim, theme.primary, theme.name)
                    .await
                {
                    self.artwork_protocol = Some(self.artwork_converter.create_protocol(img));
                }
                self.is_loading_artwork = false;
            } else {
                self.artwork_protocol = None;
                self.is_loading_artwork = false;
            }
        }
        Ok(())
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.current_theme();

    f.render_widget(Block::default().style(Style::default().bg(COLOR_BG)), area);

    let chassis_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.dim))
        .title(vec![
            Span::styled(" + ", Style::default().fg(theme.dim)),
            Span::styled(
                format!(" ❖ MODEL: AM-2026-TUI // REV: 1.0.4 // THEME: {} ", theme.name.to_uppercase()),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" + ", Style::default().fg(theme.dim)),
        ])
        .title_alignment(Alignment::Center)
        .title_bottom(vec![
            Span::styled(" + ", Style::default().fg(theme.dim)),
            Span::styled(" INDUSTRIAL AUDIO COMPONENT ", Style::default().fg(theme.dim).add_modifier(Modifier::DIM)),
            Span::styled(" + ", Style::default().fg(theme.dim)),
        ])
        .title_alignment(Alignment::Center);

    let chassis_inner = chassis_block.inner(area);
    f.render_widget(chassis_block, area);

    for y in (chassis_inner.top()..chassis_inner.bottom()).step_by(2) {
        let line = Paragraph::new(" ".repeat(chassis_inner.width as usize))
            .style(Style::default().bg(Color::Rgb(5, 5, 5)).add_modifier(Modifier::DIM));
        f.render_widget(line, ratatui::layout::Rect::new(chassis_inner.left(), y, chassis_inner.width, 1));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(chassis_inner);

    let display_area = chunks[1];
    let tuner_area = chunks[3];
    let control_area = chunks[4];

    let screen_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.dim));

    let screen_inner = screen_block.inner(display_area);
    f.render_widget(screen_block, display_area);

    let show_artwork = display_area.width > 50;

    let content_layout = if show_artwork {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Percentage(60),
            ])
            .split(screen_inner)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(screen_inner)
    };

    if show_artwork {
        let artwork_chunk = content_layout[0].inner(ratatui::layout::Margin { horizontal: 2, vertical: 1 });

        let w = artwork_chunk.width;
        let h = artwork_chunk.height;
        let size = w.min(h * 2);

        let center_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((h - size / 2).saturating_sub(1) / 2),
                Constraint::Length(size / 2),
                Constraint::Min(0),
            ])
            .split(artwork_chunk);

        let art_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((w - size) / 2),
                Constraint::Length(size),
                Constraint::Min(0),
            ])
            .split(center_layout[1])[1];

        if app.is_loading_artwork {
            let loader = Throbber::default()
                .throbber_set(BRAILLE_SIX_DOUBLE)
                .use_type(WhichUse::Spin)
                .style(Style::default().fg(theme.accent));
            f.render_stateful_widget(loader, art_rect, &mut app.throbber_state);
        } else if let Some(ref mut protocol) = app.artwork_protocol {
            let image = StatefulImage::default();
            f.render_stateful_widget(image, art_rect, protocol);
        } else {
            let no_sig = Paragraph::new("NO SIGNAL")
                .style(Style::default().fg(theme.dim).add_modifier(Modifier::DIM))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            let v_center = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(45),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(artwork_chunk);
            f.render_widget(no_sig, v_center[1]);
        }
    }

    let info_chunk = if show_artwork {
        content_layout[2]
    } else {
        content_layout[0]
    };

    if let Some(track) = app.get_current_track() {
        let status_line = Line::from(vec![
            Span::styled("SYS.STATUS: ", Style::default().fg(theme.dim)),
            Span::styled(
                "ONLINE",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("AUDIO: ", Style::default().fg(theme.dim)),
            Span::styled("PCM 44.1kHz", Style::default().fg(theme.accent)),
            Span::raw("  "),
            Span::styled("CH: ", Style::default().fg(theme.dim)),
            Span::styled("STEREO", Style::default().fg(theme.accent)),
        ]);

        let track_details = vec![
            Line::from(""),
            status_line,
            Line::from(vec![
                Span::raw("──────────────────────────────────────").fg(theme.dim)
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "TRACK TITLE",
                Style::default()
                    .fg(theme.dim)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(Span::styled(
                format!(" {} ", track.name.to_uppercase()),
                Style::default()
                    .bg(theme.dim)
                    .fg(COLOR_BG)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "ARTIST",
                Style::default()
                    .fg(theme.dim)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(Span::styled(
                format!(" {} ", track.artist.to_uppercase()),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "ALBUM REFERENCE",
                Style::default()
                    .fg(theme.dim)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(Span::styled(
                format!(" {} ", track.album.to_uppercase()),
                Style::default().fg(theme.primary),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("TIME CODE: ", Style::default().fg(theme.dim)),
                Span::styled(
                    format!(
                        "{} / {}",
                        format_duration(track.position),
                        format_duration(track.duration)
                    ),
                    Style::default()
                        .fg(theme.alert)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let info_p = Paragraph::new(track_details)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 1, 1)));

        f.render_widget(info_p, info_chunk);
    } else {
        let idle_text = vec![
            Line::from(""),
            Line::from("WAITING FOR MEDIA INPUT..."),
            Line::from(""),
            Line::from(Span::styled(
                "INSERT TAPE OR DISC",
                Style::default()
                    .fg(theme.alert)
                    .add_modifier(Modifier::SLOW_BLINK),
            )),
        ];
        let idle_p = Paragraph::new(idle_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.dim))
            .block(Block::default().padding(ratatui::widgets::Padding::new(0, 0, 5, 0)));
        f.render_widget(idle_p, info_chunk);
    }

    if let Some(track) = app.get_current_track() {
        let progress_percent = if track.duration.as_secs() > 0 {
            ((track.position.as_secs_f64() / track.duration.as_secs_f64()) * 100.0) as u16
        } else {
            0
        };

        let label = format!("| {:02}% | FREQ.TUNER :: ACTIVE |", progress_percent);
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.dim))
                    .title(vec![
                        Span::styled(" [ ", Style::default().fg(theme.dim)),
                        Span::styled("SIGNAL STRENGTH MONITOR", Style::default().fg(theme.dim)),
                        Span::styled(" ] ", Style::default().fg(theme.dim)),
                    ]),
            )
            .gauge_style(
                Style::default()
                    .fg(theme.primary)
                    .bg(Color::Rgb(15, 15, 15)),
            )
            .percent(progress_percent.min(100))
            .label(Span::styled(
                label,
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ));

        f.render_widget(gauge, tuner_area);
    }

    let controls = vec![
        ("PLAY", "SPC"),
        ("SKIP", "]"),
        ("PREV", "["),
        ("VOL+", "+"),
        ("VOL-", "-"),
        ("MUTE", "m"),
        ("EXIT", "q"),
    ];

    let btn_width = control_area.width / controls.len() as u16;
    let btn_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(btn_width); controls.len()])
        .split(control_area);

    for (i, (label, key)) in controls.iter().enumerate() {
        if i < btn_layout.len() {
            let btn_text = Line::from(vec![
                Span::styled(format!(" {}", label), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" [{}] ", key), Style::default().fg(theme.dim)),
            ]);

            let btn = Paragraph::new(btn_text).alignment(Alignment::Center).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(theme.dim))
                    .bg(Color::Rgb(10, 10, 10)),
            );

            f.render_widget(btn, btn_layout[i]);
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{MediaPlayer, PlaybackState, RepeatMode, Track};
    use async_trait::async_trait;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    struct MockPlayer {
        volume: u8,
    }

    #[async_trait]
    impl MediaPlayer for MockPlayer {
        async fn play(&self) -> Result<()> {
            Ok(())
        }
        async fn pause(&self) -> Result<()> {
            Ok(())
        }
        async fn toggle(&self) -> Result<()> {
            Ok(())
        }
        async fn next(&self) -> Result<()> {
            Ok(())
        }
        async fn previous(&self) -> Result<()> {
            Ok(())
        }
        async fn stop(&self) -> Result<()> {
            Ok(())
        }
        async fn get_current_track(&self) -> Result<Option<Track>> {
            Ok(Some(Track {
                name: "Test Song".into(),
                artist: "Test Artist".into(),
                album: "Test Album".into(),
                duration: Duration::from_secs(300),
                position: Duration::from_secs(150),
            }))
        }
        async fn get_playback_state(&self) -> Result<PlaybackState> {
            Ok(PlaybackState::Playing)
        }
        async fn set_volume(&self, _volume: u8) -> Result<()> {
            Ok(())
        }
        async fn get_volume(&self) -> Result<u8> {
            Ok(self.volume)
        }
        async fn seek(&self, _seconds: i32) -> Result<()> {
            Ok(())
        }
        async fn set_shuffle(&self, _enabled: bool) -> Result<()> {
            Ok(())
        }
        async fn set_repeat(&self, _mode: RepeatMode) -> Result<()> {
            Ok(())
        }
        async fn get_artwork_url(&self) -> Result<Option<String>> {
            Ok(Some("http://example.com/artwork.jpg".into()))
        }
    }

    #[tokio::test]
    async fn test_app_initialization() {
        let player = Box::new(MockPlayer { volume: 70 });
        let app = App::with_player(player).await.unwrap();
        assert_eq!(app.get_volume(), 70);
        assert!(!app.is_muted());
    }

    #[tokio::test]
    async fn test_ui_rendering() {
        let player = Box::new(MockPlayer { volume: 70 });
        let mut app = App::with_player(player).await.unwrap();
        app.update().await.unwrap();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| draw(f, &mut app)).unwrap();

        let buffer = terminal.backend().buffer();
        let content = format!("{:?}", buffer).to_uppercase();
        assert!(content.contains("TEST"));
        assert!(content.contains("SONG"));
        assert!(content.contains("ARTIST"));
    }
}
