use anyhow::{Context, Result};
use mpd::State as MpdPlayState;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct SongInfo {
    pub id: u32,
    pub pos: usize,
    pub file: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct MpdStatus {
    pub state: PlaybackState,
    pub volume: i8,
    pub repeat: bool,
    pub random: bool,
    pub single: bool,
    pub elapsed: Option<Duration>,
    pub total: Option<Duration>,
    pub current_song: Option<SongInfo>,
}

impl Default for MpdStatus {
    fn default() -> Self {
        Self {
            state: PlaybackState::Disconnected,
            volume: -1,
            repeat: false,
            random: false,
            single: false,
            elapsed: None,
            total: None,
            current_song: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

pub enum MpdCommand {
    TogglePause,
    Next,
    Prev,
    ChangeVolume(i8),
    PlayIndex(usize),
    DeleteIndex(u32),
    ClearQueue,
    AddPath(String),
    ImmersionMode(Option<String>),
    ToggleSingle,
    ToggleRandom,
    ToggleRepeat,
    SeekRelative(i64),
}

pub struct MpdController {
    addr: String,
    client: Option<mpd::Client>,
}

impl MpdController {
    pub fn new(addr: String) -> Self {
        Self { addr, client: None }
    }

    fn ensure_connected(&mut self) -> bool {
        if self.client.is_some() {
            return true;
        }

        match mpd::Client::connect(&self.addr) {
            Ok(client) => {
                self.client = Some(client);
                true
            }
            Err(_) => false,
        }
    }

    pub fn fetch_status(&mut self) -> MpdStatus {
        if !self.ensure_connected() {
            return MpdStatus::default();
        }

        let client = match self.client.as_mut() {
            Some(c) => c,
            None => return MpdStatus::default(),
        };

        let status = match client.status() {
            Ok(s) => s,
            Err(_) => {
                self.client = None;
                return MpdStatus::default();
            }
        };

        let state = match status.state {
            MpdPlayState::Play => PlaybackState::Playing,
            MpdPlayState::Pause => PlaybackState::Paused,
            MpdPlayState::Stop => PlaybackState::Stopped,
        };

        let current_song = match client.currentsong() {
            Ok(Some(song)) => {
                let title = get_tag(&song.tags, "Title").unwrap_or_else(|| get_filename_fallback(&song.file));
                let artist = get_tag(&song.tags, "Artist").unwrap_or_else(|| "Unknown Artist".into());
                let album = get_tag(&song.tags, "Album").unwrap_or_else(|| "Unknown Album".into());

                Some(SongInfo {
                    id: song.place.as_ref().map(|p| p.id.0 as u32).unwrap_or(0),
                    pos: song.place.as_ref().map(|p| p.pos as usize).unwrap_or(0),
                    file: song.file.clone(),
                    title,
                    artist,
                    album,
                    duration: song.duration,
                })
            }
            _ => None,
        };

        MpdStatus {
            state,
            volume: status.volume,
            repeat: status.repeat,
            random: status.random,
            single: status.single,
            elapsed: status.elapsed,
            total: status.duration,
            current_song,
        }
    }

    pub fn fetch_queue(&mut self) -> Vec<SongInfo> {
        if !self.ensure_connected() {
            return Vec::new();
        }

        let client = match self.client.as_mut() {
            Some(c) => c,
            None => return Vec::new(),
        };

        match client.queue() {
            Ok(songs) => songs
                .into_iter()
                .map(|song| {
                    let title = get_tag(&song.tags, "Title").unwrap_or_else(|| get_filename_fallback(&song.file));
                    let artist = get_tag(&song.tags, "Artist").unwrap_or_else(|| "Unknown Artist".into());
                    let album = get_tag(&song.tags, "Album").unwrap_or_else(|| "Unknown Album".into());

                    SongInfo {
                        id: song.place.as_ref().map(|p| p.id.0 as u32).unwrap_or(0),
                        pos: song.place.as_ref().map(|p| p.pos as usize).unwrap_or(0),
                        file: song.file.clone(),
                        title,
                        artist,
                        album,
                        duration: song.duration,
                    }
                })
                .collect(),
            Err(_) => {
                self.client = None;
                Vec::new()
            }
        }
    }

    pub fn fetch_directory(&mut self, path: &str, show_hidden: bool) -> Vec<DirEntry> {
        if !self.ensure_connected() {
            return Vec::new();
        }

        let client = match self.client.as_mut() {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut entries = Vec::new();
        if let Ok(list) = client.listfiles(path) {
            for (key, val) in list {
                let is_dir = key == "directory";
                let is_file = key == "file";

                if is_dir || is_file {
                    let name = std::path::Path::new(&val)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| val.clone());

                    // Filter out hidden files/folders starting with '.' unless show_hidden is true
                    if !show_hidden && name.starts_with('.') {
                        continue;
                    }

                    // Construct full relative path for MPD add command
                    let full_path = if path.is_empty() || val.starts_with(path) {
                        val.clone()
                    } else {
                        format!("{}/{}", path.trim_end_matches('/'), val)
                    };

                    entries.push(DirEntry {
                        name,
                        path: full_path,
                        is_dir,
                    });
                }
            }
        }

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        entries
    }

    pub fn push_path_recursive(&mut self, path: &str) -> Result<()> {
        if !self.ensure_connected() {
            anyhow::bail!("MPD not connected");
        }

        let initial_queue_len = self.fetch_queue().len();

        // 1. Try pushing path directly to MPD
        if let Some(client) = self.client.as_mut() {
            let song = mpd::Song {
                file: path.to_string(),
                ..Default::default()
            };
            let _ = client.push(song);
        }

        let new_len = self.fetch_queue().len();

        // 2. If queue length grew, direct push worked!
        if new_len > initial_queue_len {
            return Ok(());
        }

        // 3. Otherwise, inspect directory entries and push files individually
        let entries = self.fetch_directory(path, false);
        for entry in entries {
            if entry.is_dir {
                let _ = self.push_path_recursive(&entry.path);
            } else if let Some(client) = self.client.as_mut() {
                let song = mpd::Song {
                    file: entry.path,
                    ..Default::default()
                };
                let _ = client.push(song);
            }
        }

        Ok(())
    }

    pub fn execute(&mut self, cmd: MpdCommand) -> Result<()> {
        if !self.ensure_connected() {
            anyhow::bail!("MPD not connected");
        }

        match cmd {
            MpdCommand::TogglePause => {
                let client = self.client.as_mut().context("No MPD client")?;
                let st = client.status()?;
                match st.state {
                    MpdPlayState::Play => client.pause(true)?,
                    MpdPlayState::Pause => client.pause(false)?,
                    MpdPlayState::Stop => client.play()?,
                }
            }
            MpdCommand::Next => {
                if let Some(client) = self.client.as_mut() {
                    let _ = client.next();
                }
            }
            MpdCommand::Prev => {
                if let Some(client) = self.client.as_mut() {
                    let _ = client.prev();
                }
            }
            MpdCommand::ChangeVolume(delta) => {
                if let Some(client) = self.client.as_mut() {
                    let current = client.status().map(|s| s.volume).unwrap_or(50);
                    let new_vol = (current + delta).clamp(0, 100);
                    let _ = client.volume(new_vol);
                }
            }
            MpdCommand::PlayIndex(pos) => {
                if let Some(client) = self.client.as_mut() {
                    let _ = client.switch(pos as u32);
                }
            }
            MpdCommand::DeleteIndex(id) => {
                if let Some(client) = self.client.as_mut() {
                    let _ = client.delete(mpd::Id(id));
                }
            }
            MpdCommand::ClearQueue => {
                if let Some(client) = self.client.as_mut() {
                    let _ = client.clear();
                }
            }
            MpdCommand::AddPath(path) => {
                self.push_path_recursive(&path)?;
            }
            MpdCommand::ImmersionMode(target) => {
                if let Some(client) = self.client.as_mut() {
                    let _ = client.clear();
                }

                let path = target.unwrap_or_else(|| "".into());
                let target_path = if path == "/" { "".into() } else { path };

                self.push_path_recursive(&target_path)?;

                if let Some(client) = self.client.as_mut() {
                    let _ = client.random(true);
                    let _ = client.repeat(true);
                    let _ = client.play();
                }
            }
            MpdCommand::ToggleSingle => {
                if let Some(client) = self.client.as_mut() {
                    let current = client.status().map(|s| s.single).unwrap_or(false);
                    let _ = client.single(!current);
                }
            }
            MpdCommand::ToggleRandom => {
                if let Some(client) = self.client.as_mut() {
                    let current = client.status().map(|s| s.random).unwrap_or(false);
                    let _ = client.random(!current);
                }
            }
            MpdCommand::ToggleRepeat => {
                if let Some(client) = self.client.as_mut() {
                    let current = client.status().map(|s| s.repeat).unwrap_or(false);
                    let _ = client.repeat(!current);
                }
            }
            MpdCommand::SeekRelative(seconds) => {
                if let Some(client) = self.client.as_mut() {
                    if let Ok(st) = client.status() {
                        if let Some(elapsed) = st.elapsed {
                            let new_secs = (elapsed.as_secs() as i64 + seconds).max(0);
                            if let Some(place) = st.song {
                                let _ = client.seek(place.pos as u32, new_secs as i64);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn get_tag(tags: &[(String, String)], key: &str) -> Option<String> {
    tags.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v.clone())
}

fn get_filename_fallback(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}
