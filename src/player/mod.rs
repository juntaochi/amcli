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

#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub track: Option<Track>,
    pub volume: u8,
    #[allow(dead_code)]
    pub state: PlaybackState,
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

    async fn set_volume(&self, volume: u8) -> Result<()>;
    async fn get_volume(&self) -> Result<u8>;
    async fn seek(&self, seconds: i32) -> Result<()>;
    #[allow(dead_code)]
    async fn set_shuffle(&self, enabled: bool) -> Result<()>;
    async fn set_repeat(&self, mode: RepeatMode) -> Result<()>;
    async fn get_artwork_url(&self, track: &Track) -> Result<Option<String>>;

    // ⚡ Bolt: Provide a fallback that concurrently fetches the required properties.
    // It purposefully avoids calling `get_playback_state` to prevent unnecessary overhead
    // for default players, as the UI currently doesn't use the state field.
    async fn get_player_status(&self) -> Result<PlayerStatus> {
        let (track_result, volume_result) =
            tokio::join!(self.get_current_track(), self.get_volume(),);

        Ok(PlayerStatus {
            track: track_result.ok().flatten(),
            // Ensure volume defaults correctly when it fails (or returns a dummy value)
            volume: volume_result.unwrap_or(50),
            state: PlaybackState::Stopped,
        })
    }
}
