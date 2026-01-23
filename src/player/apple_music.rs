// src/player/apple_music.rs
use super::{MediaPlayer, PlaybackState, RepeatMode, Track};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::time::Duration;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn execute(&self, script: &str) -> Result<std::process::Output>;
}

pub struct OsascriptRunner;

#[async_trait]
impl CommandRunner for OsascriptRunner {
    async fn execute(&self, script: &str) -> Result<std::process::Output> {
        tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .await
            .map_err(|e| anyhow!(e))
    }
}

pub struct AppleMusicController {
    runner: Box<dyn CommandRunner>,
}

impl AppleMusicController {
    pub fn new() -> Self {
        Self {
            runner: Box::new(OsascriptRunner),
        }
    }

    #[cfg(test)]
    pub fn with_runner(runner: Box<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    async fn execute_script(&self, script: &str) -> Result<String> {
        let output = self.runner.execute(script).await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(anyhow!(
                "AppleScript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

#[async_trait]
impl MediaPlayer for AppleMusicController {
    async fn play(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to play"#).await?;
        Ok(())
    }

    async fn pause(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to pause"#).await?;
        Ok(())
    }

    async fn toggle(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to playpause"#).await?;
        Ok(())
    }

    async fn next(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to next track"#).await?;
        Ok(())
    }

    async fn previous(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to previous track"#).await?;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to stop"#).await?;
        Ok(())
    }

    async fn get_current_track(&self) -> Result<Option<Track>> {
        let script = r#"
            tell application "Music"
                if player state is not stopped then
                    set output to name of current track & "|" & ¬
                                  artist of current track & "|" & ¬
                                  album of current track & "|" & ¬
                                  duration of current track & "|" & ¬
                                  player position
                    return output
                else
                    return ""
                end if
            end tell
        "#;

        let result = self.execute_script(script).await?;

        if result.is_empty() {
            return Ok(None);
        }

        let parts: Vec<&str> = result.split('|').collect();
        if parts.len() < 5 {
            return Err(anyhow!("Invalid track info format"));
        }

        Ok(Some(Track {
            name: parts[0].to_string(),
            artist: parts[1].to_string(),
            album: parts[2].to_string(),
            duration: Duration::from_secs_f64(parts[3].parse()?),
            position: Duration::from_secs_f64(parts[4].parse()?),
        }))
    }

    async fn get_playback_state(&self) -> Result<PlaybackState> {
        let script = r#"tell application "Music" to return player state as string"#;
        let state = self.execute_script(script).await?;

        match state.as_str() {
            "playing" => Ok(PlaybackState::Playing),
            "paused" => Ok(PlaybackState::Paused),
            "stopped" => Ok(PlaybackState::Stopped),
            _ => Err(anyhow!("Unknown playback state: {}", state)),
        }
    }

    async fn set_volume(&self, volume: u8) -> Result<()> {
        let script = format!(
            r#"tell application "Music" to set sound volume to {}"#,
            volume
        );
        self.execute_script(&script).await?;
        Ok(())
    }

    async fn get_volume(&self) -> Result<u8> {
        let script = r#"tell application "Music" to return sound volume"#;
        let volume = self.execute_script(script).await?;
        Ok(volume.parse()?)
    }

    async fn seek(&self, seconds: i32) -> Result<()> {
        let script = format!(
            r#"tell application "Music" to set player position to (player position + {})"#,
            seconds
        );
        self.execute_script(&script).await?;
        Ok(())
    }

    async fn set_shuffle(&self, enabled: bool) -> Result<()> {
        let script = format!(
            r#"tell application "Music" to set shuffle enabled to {}"#,
            enabled
        );
        self.execute_script(&script).await?;
        Ok(())
    }

    async fn set_repeat(&self, mode: RepeatMode) -> Result<()> {
        let mode_str = match mode {
            RepeatMode::Off => "off",
            RepeatMode::One => "one",
            RepeatMode::All => "all",
        };
        let script = format!(
            r#"tell application "Music" to set song repeat to {}"#,
            mode_str
        );
        self.execute_script(&script).await?;
        Ok(())
    }

    async fn get_artwork_url(&self) -> Result<Option<String>> {
        let track = match self.get_current_track().await? {
            Some(t) => t,
            None => return Ok(None),
        };

        let query = format!("{} {}", track.artist, track.name);
        let url = format!(
            "https://itunes.apple.com/search?term={}&entity=song&limit=1",
            urlencoding::encode(&query)
        );

        let timeout_duration = std::time::Duration::from_secs(3);
        let response = tokio::time::timeout(
            timeout_duration,
            reqwest::get(url)
        ).await??;
        
        let json = tokio::time::timeout(
            timeout_duration,
            response.json::<serde_json::Value>()
        ).await??;
        
        let artwork_url = json["results"][0]["artworkUrl100"]
            .as_str()
            .map(|s| s.replace("100x100bb", "600x600bb"));

        Ok(artwork_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    fn mock_output(stdout: &str, success: bool) -> std::process::Output {
        std::process::Output {
            status: ExitStatus::from_raw(if success { 0 } else { 1 }),
            stdout: stdout.as_bytes().to_vec(),
            stderr: if success { vec![] } else { b"error".to_vec() },
        }
    }

    #[tokio::test]
    async fn test_play() {
        let mut mock = MockCommandRunner::new();
        mock.expect_execute()
            .with(mockall::predicate::eq(
                r#"tell application "Music" to play"#,
            ))
            .times(1)
            .returning(|_| Ok(mock_output("", true)));

        let controller = AppleMusicController::with_runner(Box::new(mock));
        assert!(controller.play().await.is_ok());
    }

    #[tokio::test]
    async fn test_get_volume() {
        let mut mock = MockCommandRunner::new();
        mock.expect_execute()
            .with(mockall::predicate::eq(
                r#"tell application "Music" to return sound volume"#,
            ))
            .times(1)
            .returning(|_| Ok(mock_output("75", true)));

        let controller = AppleMusicController::with_runner(Box::new(mock));
        let volume = controller.get_volume().await.unwrap();
        assert_eq!(volume, 75);
    }

    #[tokio::test]
    async fn test_get_current_track() {
        let mut mock = MockCommandRunner::new();
        let output = "Song Name|Artist Name|Album Name|180.5|90.0";
        mock.expect_execute()
            .times(1)
            .returning(move |_| Ok(mock_output(output, true)));

        let controller = AppleMusicController::with_runner(Box::new(mock));
        let track = controller.get_current_track().await.unwrap().unwrap();
        assert_eq!(track.name, "Song Name");
        assert_eq!(track.artist, "Artist Name");
        assert_eq!(track.duration.as_secs(), 180);
        assert_eq!(track.position.as_secs(), 90);
    }
}
