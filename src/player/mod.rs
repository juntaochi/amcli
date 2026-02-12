// src/player/mod.rs
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

pub mod apple_music;

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    pub position: Duration,
}

#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub track: Option<Track>,
    pub volume: Option<u8>,
    #[allow(dead_code)]
    pub state: Option<PlaybackState>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

#[async_trait]
pub trait MediaPlayer: Send + Sync {
    #[allow(dead_code)]
    async fn play(&self) -> Result<()>;
    #[allow(dead_code)]
    async fn pause(&self) -> Result<()>;
    async fn toggle(&self) -> Result<()>;
    async fn next(&self) -> Result<()>;
    async fn previous(&self) -> Result<()>;
    #[allow(dead_code)]
    async fn stop(&self) -> Result<()>;

    async fn get_current_track(&self) -> Result<Option<Track>>;
    #[allow(dead_code)]
    async fn get_playback_state(&self) -> Result<PlaybackState>;

    async fn get_player_status(&self) -> Result<PlayerStatus> {
        let (track, volume, state) = tokio::join!(
            self.get_current_track(),
            self.get_volume(),
            self.get_playback_state()
        );
        Ok(PlayerStatus {
            track: track.unwrap_or(None),
            volume: volume.ok(),
            state: state.ok(),
        })
    }

    async fn set_volume(&self, volume: u8) -> Result<()>;
    async fn get_volume(&self) -> Result<u8>;
    async fn seek(&self, seconds: i32) -> Result<()>;
    async fn set_shuffle(&self, enabled: bool) -> Result<()>;
    async fn set_repeat(&self, mode: RepeatMode) -> Result<()>;
    async fn get_artwork_url(&self, track: &Track) -> Result<Option<String>>;
}
