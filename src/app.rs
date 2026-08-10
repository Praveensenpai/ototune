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
    pub notification: Option<(String, Instant)>,
    pub running: bool,

    pub persistent_state: PersistentState,
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
            notification: None,
            running: true,

            persistent_state: persistent,
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
