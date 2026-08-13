use crate::mpd_client::{DirEntry, MpdStatus, SongInfo};
use crate::state::PersistentState;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Playlist,
    Browser,
}

pub struct AppState {
    pub active_tab: ActiveTab,
    pub status: MpdStatus,
    pub queue: Vec<SongInfo>,
    pub playlist_selected: usize,

    pub browser_path: String,
    pub browser_entries: Vec<DirEntry>,
    pub browser_selected: usize,
    pub browser_history: Vec<String>,
    pub show_hidden: bool,
    pub resume_mode: bool,

    pub search_mode: bool,
    pub search_query: String,
    pub filtered_queue_indices: Vec<usize>,

    pub show_help: bool,
    pub show_archive_settings: bool,
    pub archive_settings_selected: usize,

    pub notification: Option<(String, Instant)>,
    pub running: bool,

    pub persistent_state: PersistentState,
    pub last_tracked_song: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let persistent = PersistentState::load();

        Self {
            active_tab: ActiveTab::Playlist,
            status: MpdStatus::default(),
            queue: Vec::new(),
            playlist_selected: 0,

            browser_path: persistent.last_folder.clone(),
            browser_entries: Vec::new(),
            browser_selected: 0,
            browser_history: Vec::new(),
            show_hidden: persistent.show_hidden,
            resume_mode: persistent.resume_mode,

            search_mode: false,
            search_query: String::new(),
            filtered_queue_indices: Vec::new(),

            show_help: false,
            show_archive_settings: false,
            archive_settings_selected: 0,

            notification: None,
            running: true,

            persistent_state: persistent,
            last_tracked_song: None,
        }
    }

    pub fn check_auto_archive(&mut self, controller: &mut crate::mpd_client::MpdController, delta_secs: f64) {
        let current_file = self.status.current_song.as_ref().map(|s| s.file.clone());

        // 1. Detect track transition: when track finishes or user switches tracks
        if self.last_tracked_song != current_file {
            if let Some(prev_file) = self.last_tracked_song.take() {
                let accum_secs = self.persistent_state.listen_times_secs.get(&prev_file).copied().unwrap_or(0.0);
                
                let total_dur = self.queue.iter().find(|s| s.file == prev_file)
                    .and_then(|s| s.duration)
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);

                if total_dur > 0.0 {
                    let req_listens = self.persistent_state.auto_archive.required_listens as f64;
                    let comp_ratio = self.persistent_state.auto_archive.completion_percent as f64 / 100.0;
                    let total_req_secs = req_listens * comp_ratio * total_dur;

                    if accum_secs >= total_req_secs && self.persistent_state.auto_archive.enabled {
                        // Track completed listening quota! Archive now that it finished playing.
                        if self.archive_file_on_disk(&prev_file, controller) {
                            self.persistent_state.listen_times_secs.remove(&prev_file);
                            self.persistent_state.play_counts.remove(&prev_file);
                            self.set_notification(format!(
                                "📦 Auto-archived completed track: '{}'",
                                get_filename_fallback(&prev_file)
                            ));
                        }
                    }
                }
            }
            self.last_tracked_song = current_file.clone();
        }

        // 2. Accumulate actual wall-clock playback duration while track is playing
        if self.status.state == crate::mpd_client::PlaybackState::Playing {
            if let Some(song) = self.status.current_song.clone() {
                let file = song.file.clone();
                let total_duration_secs = song.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);

                if total_duration_secs > 0.0 {
                    let entry = self.persistent_state.listen_times_secs.entry(file.clone()).or_insert(0.0);
                    *entry += delta_secs;
                    let accumulated_secs = *entry;

                    let _req_listens = self.persistent_state.auto_archive.required_listens as f64;
                    let completion_ratio = self.persistent_state.auto_archive.completion_percent as f64 / 100.0;
                    let single_listen_req = completion_ratio * total_duration_secs;

                    // Update estimated play count badge
                    let current_plays = (accumulated_secs / single_listen_req).floor() as u32;
                    self.persistent_state.play_counts.insert(file.clone(), current_plays);
                }
            }
        }
    }

    fn archive_file_on_disk(&mut self, rel_path: &str, controller: &mut crate::mpd_client::MpdController) -> bool {
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return false,
        };
        let music_base = std::path::PathBuf::from(home).join("Music").join("immersionpod");
        let src_path = music_base.join(rel_path);

        if !src_path.exists() {
            return false;
        }

        let file_name = match src_path.file_name() {
            Some(n) => n,
            None => return false,
        };

        let archive_dir_name = &self.persistent_state.auto_archive.archive_dir;
        let archive_dir = music_base.join(archive_dir_name);
        if let Err(_) = std::fs::create_dir_all(&archive_dir) {
            return false;
        }

        let dst_path = archive_dir.join(file_name);

        // Try renaming or fallback to copy+delete
        let moved = if std::fs::rename(&src_path, &dst_path).is_ok() {
            true
        } else if std::fs::copy(&src_path, &dst_path).is_ok() {
            let _ = std::fs::remove_file(&src_path);
            true
        } else {
            false
        };

        if moved {
            let _ = controller.execute(crate::mpd_client::MpdCommand::UpdateDb);
            true
        } else {
            false
        }
    }

    pub fn save_current_state(&mut self) {
        self.persistent_state.last_folder = self.browser_path.clone();
        self.persistent_state.show_hidden = self.show_hidden;
        self.persistent_state.resume_mode = self.resume_mode;

        if let Some(song) = &self.status.current_song {
            self.persistent_state.last_file = Some(song.file.clone());
        }

        if let Some(elapsed) = self.status.elapsed {
            self.persistent_state.last_elapsed_secs = elapsed.as_secs();
        }

        self.persistent_state.save();
    }

    pub fn set_notification(&mut self, msg: impl Into<String>) {
        self.notification = Some((msg.into(), Instant::now()));
    }

    pub fn get_active_notification(&self) -> Option<&str> {
        if let Some((msg, created)) = &self.notification {
            if created.elapsed() < Duration::from_secs(3) {
                return Some(msg.as_str());
            }
        }
        None
    }

    pub fn update_filtered_indices(&mut self) {
        if self.search_query.trim().is_empty() {
            self.filtered_queue_indices = (0..self.queue.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_queue_indices = self
                .queue
                .iter()
                .enumerate()
                .filter(|(_, song)| {
                    song.title.to_lowercase().contains(&query)
                        || song.artist.to_lowercase().contains(&query)
                        || song.album.to_lowercase().contains(&query)
                        || song.file.to_lowercase().contains(&query)
                })
                .map(|(idx, _)| idx)
                .collect();
        }

        if self.playlist_selected >= self.filtered_queue_indices.len() {
            self.playlist_selected = self.filtered_queue_indices.len().saturating_sub(1);
        }
    }

    pub fn selected_song_index(&self) -> Option<usize> {
        if self.search_mode || !self.search_query.is_empty() {
            self.filtered_queue_indices
                .get(self.playlist_selected)
                .copied()
        } else if self.playlist_selected < self.queue.len() {
            Some(self.playlist_selected)
        } else {
            None
        }
    }

    pub fn next_item(&mut self) {
        match self.active_tab {
            ActiveTab::Playlist => {
                let count = if self.search_mode || !self.search_query.is_empty() {
                    self.filtered_queue_indices.len()
                } else {
                    self.queue.len()
                };
                if count > 0 && self.playlist_selected + 1 < count {
                    self.playlist_selected += 1;
                }
            }
            ActiveTab::Browser => {
                if !self.browser_entries.is_empty()
                    && self.browser_selected + 1 < self.browser_entries.len()
                {
                    self.browser_selected += 1;
                }
            }
        }
    }

    pub fn prev_item(&mut self) {
        match self.active_tab {
            ActiveTab::Playlist => {
                if self.playlist_selected > 0 {
                    self.playlist_selected -= 1;
                }
            }
            ActiveTab::Browser => {
                if self.browser_selected > 0 {
                    self.browser_selected -= 1;
                }
            }
        }
    }

    pub fn page_down(&mut self) {
        match self.active_tab {
            ActiveTab::Playlist => {
                let count = if self.search_mode || !self.search_query.is_empty() {
                    self.filtered_queue_indices.len()
                } else {
                    self.queue.len()
                };
                if count > 0 {
                    self.playlist_selected = (self.playlist_selected + 10).min(count - 1);
                }
            }
            ActiveTab::Browser => {
                if !self.browser_entries.is_empty() {
                    self.browser_selected =
                        (self.browser_selected + 10).min(self.browser_entries.len() - 1);
                }
            }
        }
    }

    pub fn page_up(&mut self) {
        match self.active_tab {
            ActiveTab::Playlist => {
                self.playlist_selected = self.playlist_selected.saturating_sub(10);
            }
            ActiveTab::Browser => {
                self.browser_selected = self.browser_selected.saturating_sub(10);
            }
        }
    }
}

fn get_filename_fallback(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

