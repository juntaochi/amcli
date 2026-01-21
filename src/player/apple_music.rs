// src/player/apple_music.rs
use super::{MediaPlayer, PlaybackState, RepeatMode, Track};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::process::{Command, Output};
use std::time::Duration;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait CommandRunner: Send + Sync {
    fn execute(&self, script: &str) -> Result<Output>;
}

pub struct OsascriptRunner;

impl CommandRunner for OsascriptRunner {
    fn execute(&self, script: &str) -> Result<Output> {
        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
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

    fn execute_script(&self, script: &str) -> Result<String> {
        let output = self.runner.execute(script)?;

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
        self.execute_script(r#"tell application "Music" to play"#)?;
        Ok(())
    }

    async fn pause(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to pause"#)?;
        Ok(())
    }

    async fn toggle(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to playpause"#)?;
        Ok(())
    }

    async fn next(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to next track"#)?;
        Ok(())
    }

    async fn previous(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to previous track"#)?;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.execute_script(r#"tell application "Music" to stop"#)?;
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

        let result = self.execute_script(script)?;

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
        let state = self.execute_script(script)?;

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
        self.execute_script(&script)?;
        Ok(())
    }

    async fn get_volume(&self) -> Result<u8> {
        let script = r#"tell application "Music" to return sound volume"#;
        let volume = self.execute_script(script)?;
        Ok(volume.parse()?)
    }

    async fn seek(&self, seconds: i32) -> Result<()> {
        let script = format!(
            r#"tell application "Music" to set player position to (player position + {})"#,
            seconds
        );
        self.execute_script(&script)?;
        Ok(())
    }

    async fn set_shuffle(&self, enabled: bool) -> Result<()> {
        let script = format!(
            r#"tell application "Music" to set shuffle enabled to {}"#,
            enabled
        );
        self.execute_script(&script)?;
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
        self.execute_script(&script)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    fn mock_output(stdout: &str, success: bool) -> Output {
        Output {
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
