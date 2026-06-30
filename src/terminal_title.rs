use crate::player::Track;
use crossterm::{execute, terminal::SetTitle};
use std::io::{self, Write};

pub(crate) struct TerminalTitle {
    last_title: Option<String>,
}

impl TerminalTitle {
    pub(crate) fn new() -> Self {
        Self { last_title: None }
    }

    pub(crate) fn sync(&mut self, track: Option<&Track>) -> io::Result<bool> {
        let mut stdout = io::stdout();
        self.sync_with_writer(&mut stdout, track)
    }

    fn sync_with_writer<W: Write>(
        &mut self,
        mut writer: W,
        track: Option<&Track>,
    ) -> io::Result<bool> {
        let title = title_for_track(track);
        if self.last_title.as_deref() == Some(title.as_str()) {
            return Ok(false);
        }

        execute!(writer, SetTitle(title.as_str()))?;
        self.last_title = Some(title);
        Ok(true)
    }
}

pub(crate) fn title_for_track(track: Option<&Track>) -> String {
    match track {
        Some(track) if !track.name.trim().is_empty() => format!("AMCLI: {}", track.name.trim()),
        _ => "AMCLI".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Track;
    use std::time::Duration;

    fn track(name: &str) -> Track {
        Track {
            name: name.into(),
            artist: "Artist".into(),
            album: "Album".into(),
            duration: Duration::from_secs(180),
            position: Duration::ZERO,
        }
    }

    #[test]
    fn formats_current_track_as_amcli_title() {
        assert_eq!(title_for_track(Some(&track("小情歌"))), "AMCLI: 小情歌");
    }

    #[test]
    fn formats_missing_track_as_plain_amcli_title() {
        assert_eq!(title_for_track(None), "AMCLI");
    }

    #[test]
    fn sync_writes_title_only_when_track_title_changes() {
        let mut terminal_title = TerminalTitle::new();
        let mut output = Vec::new();

        assert!(terminal_title
            .sync_with_writer(&mut output, Some(&track("First Song")))
            .unwrap());
        let first_write_len = output.len();

        assert!(!terminal_title
            .sync_with_writer(&mut output, Some(&track("First Song")))
            .unwrap());
        assert_eq!(output.len(), first_write_len);

        assert!(terminal_title
            .sync_with_writer(&mut output, Some(&track("Second Song")))
            .unwrap());
        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("AMCLI: First Song"));
        assert!(rendered.contains("AMCLI: Second Song"));
    }
}
