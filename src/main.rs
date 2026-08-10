mod app;
mod mpd_client;
mod state;
mod ui;

use app::{ActiveTab, AppState};
use mpd_client::{MpdCommand, MpdController};
use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    env,
    io::stdout,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(name = "ototune", author = "Praveen Senpai", version = "0.1.0", about = "Minimal aesthetic Rust TUI MPD player tailored for daily listening and AJATT audio immersion")]
struct Args {
    /// MPD address (host:port or host)
    #[arg(short, long, default_value = "127.0.0.1:6600")]
    address: String,

    /// Start immediately in AJATT Immersion Mode
    #[arg(short, long)]
    immersion: bool,

    /// Show hidden files and folders (.stfolder, etc.)
    #[arg(short = 'H', long)]
    show_hidden: bool,

    /// Resume playback from last saved position
    #[arg(short, long)]
    resume: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mpd_addr = if args.address == "127.0.0.1:6600" {
        let host = env::var("MPD_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("MPD_PORT").unwrap_or_else(|_| "6600".into());
        format!("{}:{}", host, port)
    } else {
        args.address.clone()
    };

    let mut controller = MpdController::new(mpd_addr);

    if args.immersion {
        let _ = controller.execute(MpdCommand::ImmersionMode(None));
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new();
    if args.show_hidden {
        app.show_hidden = true;
    }
    if args.resume {
        app.resume_mode = true;
    }

    app.status = controller.fetch_status();
    app.queue = controller.fetch_queue();
    app.update_filtered_indices();
    app.browser_entries = controller.fetch_directory(&app.browser_path, app.show_hidden);

    // Auto-resume from last saved position if Resume Mode is enabled & queue was empty
    if app.resume_mode && app.queue.is_empty() {
        if let Some(last_file) = app.persistent_state.last_file.clone() {
            let _ = controller.execute(MpdCommand::AddPath(last_file));
            app.queue = controller.fetch_queue();
            app.update_filtered_indices();

            let last_secs = app.persistent_state.last_elapsed_secs;
            if last_secs > 0 {
                let _ = controller.execute(MpdCommand::SeekRelative(last_secs as i64));
                app.set_notification(format!("📍 Resumed playback position: {:02}:{:02}", last_secs / 60, last_secs % 60));
            } else {
                app.set_notification("📍 Resumed last track!");
            }
        }
    } else if args.immersion {
        app.set_notification("🎧 Started in AJATT Immersion Mode!");
    }

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    while app.running {
        terminal.draw(|f| ui::render_ui(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(&mut app, &mut controller, key);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.status = controller.fetch_status();
            let new_queue = controller.fetch_queue();
            if new_queue.len() != app.queue.len() {
                app.queue = new_queue;
                app.update_filtered_indices();
            } else {
                app.queue = new_queue;
            }
            app.save_current_state();
            last_tick = Instant::now();
        }
    }

    // Save final state before exiting
    app.save_current_state();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key_event(app: &mut AppState, controller: &mut MpdController, key: crossterm::event::KeyEvent) {
    if app.search_mode {
        match key.code {
            KeyCode::Esc => {
                app.search_mode = false;
            }
            KeyCode::Enter => {
                app.search_mode = false;
                if let Some(song_idx) = app.selected_song_index() {
                    let _ = controller.execute(MpdCommand::PlayIndex(song_idx));
                    app.set_notification("▶ Playing selected track");
                }
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.update_filtered_indices();
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.update_filtered_indices();
            }
            _ => {}
        }
        return;
    }

    if app.show_help {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
            app.show_help = false;
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }

        KeyCode::Char('?') => {
            app.show_help = true;
        }

        KeyCode::Tab => {
            app.active_tab = match app.active_tab {
                ActiveTab::Playlist => ActiveTab::Browser,
                ActiveTab::Browser => ActiveTab::Playlist,
            };
        }

        KeyCode::Char(' ') => {
            let _ = controller.execute(MpdCommand::TogglePause);
            app.status = controller.fetch_status();
            match app.status.state {
                mpd_client::PlaybackState::Playing => app.set_notification("▶ Playing"),
                mpd_client::PlaybackState::Paused => app.set_notification("⏸ Paused"),
                _ => {}
            }
        }
        KeyCode::Char('n') => {
            let _ = controller.execute(MpdCommand::Next);
            app.set_notification("⏭ Next Track");
        }
        KeyCode::Char('p') => {
            let _ = controller.execute(MpdCommand::Prev);
            app.set_notification("⏮ Previous Track");
        }

        KeyCode::Char('+') | KeyCode::Char('=') => {
            let _ = controller.execute(MpdCommand::ChangeVolume(5));
            app.status = controller.fetch_status();
            app.set_notification(format!("🔊 Volume: {}%", app.status.volume));
        }
        KeyCode::Char('-') => {
            let _ = controller.execute(MpdCommand::ChangeVolume(-5));
            app.status = controller.fetch_status();
            app.set_notification(format!("🔊 Volume: {}%", app.status.volume));
        }

        KeyCode::Right => {
            let _ = controller.execute(MpdCommand::SeekRelative(5));
            app.set_notification("⏩ +5s");
        }
        KeyCode::Left => {
            let _ = controller.execute(MpdCommand::SeekRelative(-5));
            app.set_notification("⏪ -5s");
        }

        // Toggle Resume Mode ('m')
        KeyCode::Char('m') => {
            app.resume_mode = !app.resume_mode;
            app.save_current_state();
            if app.resume_mode {
                app.set_notification("📍 Resume Mode Enabled (Remember exact position)");
            } else {
                app.set_notification("➡️ Resume Mode Disabled (Start from beginning)");
            }
        }

        // Toggle Random & Repeat Mode
        KeyCode::Char('r') => {
            let _ = controller.execute(MpdCommand::ToggleRandom);
            app.status = controller.fetch_status();
            if app.status.random {
                app.set_notification("🔀 Random Playback Enabled");
            } else {
                app.set_notification("➡️ Random Playback Disabled");
            }
        }
        KeyCode::Char('e') => {
            let _ = controller.execute(MpdCommand::ToggleRepeat);
            app.status = controller.fetch_status();
            if app.status.repeat {
                app.set_notification("🔁 Repeat Mode Enabled");
            } else {
                app.set_notification("➡️ Repeat Mode Disabled");
            }
        }

        // Context-Aware Immersion ('i' = active folder or current path; 'I' = global full library)
        KeyCode::Char('i') => {
            let target_path = if !app.browser_path.is_empty() {
                Some(app.browser_path.clone())
            } else if app.active_tab == ActiveTab::Browser {
                app.browser_entries
                    .get(app.browser_selected)
                    .map(|e| e.path.clone())
            } else {
                None
            };

            let notif_name = target_path
                .as_ref()
                .cloned()
                .unwrap_or_else(|| "Full Library".into());

            let _ = controller.execute(MpdCommand::ImmersionMode(target_path));
            app.queue = controller.fetch_queue();
            app.update_filtered_indices();
            app.set_notification(format!("🎧 Shuffled Immersion: '{}'", notif_name));
        }
        KeyCode::Char('I') => {
            let _ = controller.execute(MpdCommand::ImmersionMode(None));
            app.queue = controller.fetch_queue();
            app.update_filtered_indices();
            app.set_notification("🎧 Global Full Library Immersion Mode!");
        }

        KeyCode::Char('l') => {
            let _ = controller.execute(MpdCommand::ToggleSingle);
            app.status = controller.fetch_status();
            if app.status.single {
                app.set_notification("🔁 Single Track Loop Enabled");
            } else {
                app.set_notification("➡️ Single Track Loop Disabled");
            }
        }

        KeyCode::Char('.') => {
            app.show_hidden = !app.show_hidden;
            app.browser_entries = controller.fetch_directory(&app.browser_path, app.show_hidden);
            app.browser_selected = 0;
            if app.show_hidden {
                app.set_notification("👁 Showing Hidden Files");
            } else {
                app.set_notification("🙈 Hiding Hidden Files");
            }
        }

        KeyCode::Char('j') | KeyCode::Down => {
            app.next_item();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.prev_item();
        }
        KeyCode::PageDown => {
            app.page_down();
        }
        KeyCode::PageUp => {
            app.page_up();
        }

        KeyCode::Char('/') => {
            app.search_mode = true;
            app.active_tab = ActiveTab::Playlist;
        }
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.update_filtered_indices();
                app.set_notification("🔍 Search Filter Cleared");
            }
        }

        KeyCode::Enter => match app.active_tab {
            ActiveTab::Playlist => {
                if let Some(song_idx) = app.selected_song_index() {
                    let _ = controller.execute(MpdCommand::PlayIndex(song_idx));
                    app.set_notification("▶ Playing track");
                }
            }
            ActiveTab::Browser => {
                if let Some(entry) = app.browser_entries.get(app.browser_selected).cloned() {
                    if entry.is_dir {
                        app.browser_history.push(app.browser_path.clone());
                        app.browser_path = entry.path.clone();
                        app.browser_entries = controller.fetch_directory(&app.browser_path, app.show_hidden);
                        app.browser_selected = 0;
                    } else {
                        let was_empty = app.queue.is_empty();
                        let res = controller.execute(MpdCommand::AddPath(entry.path.clone()));
                        if res.is_ok() {
                            if was_empty || app.status.state == mpd_client::PlaybackState::Stopped {
                                let _ = controller.execute(MpdCommand::TogglePause);
                            }
                            app.queue = controller.fetch_queue();
                            app.update_filtered_indices();
                            app.set_notification(format!("➕ Added '{}' to Queue", entry.name));
                        } else {
                            app.set_notification(format!("❌ Failed to queue '{}'", entry.name));
                        }
                    }
                }
            }
        },

        KeyCode::Char('a') if app.active_tab == ActiveTab::Browser => {
            if let Some(entry) = app.browser_entries.get(app.browser_selected).cloned() {
                let was_empty = app.queue.is_empty();
                let res = controller.execute(MpdCommand::AddPath(entry.path));
                if res.is_ok() {
                    if was_empty || app.status.state == mpd_client::PlaybackState::Stopped {
                        let _ = controller.execute(MpdCommand::TogglePause);
                    }
                    app.queue = controller.fetch_queue();
                    app.update_filtered_indices();
                    app.set_notification(format!("➕ Queued '{}'", entry.name));
                } else {
                    app.set_notification(format!("❌ Failed to queue '{}'", entry.name));
                }
            }
        }

        KeyCode::Backspace | KeyCode::Char('h') if app.active_tab == ActiveTab::Browser => {
            if let Some(parent) = app.browser_history.pop() {
                app.browser_path = parent;
                app.browser_entries = controller.fetch_directory(&app.browser_path, app.show_hidden);
                app.browser_selected = 0;
            } else if !app.browser_path.is_empty() {
                app.browser_path.clear();
                app.browser_entries = controller.fetch_directory("", app.show_hidden);
                app.browser_selected = 0;
            }
        }

        KeyCode::Char('d') | KeyCode::Delete if app.active_tab == ActiveTab::Playlist => {
            if let Some(song_idx) = app.selected_song_index() {
                if let Some(song) = app.queue.get(song_idx) {
                    let _ = controller.execute(MpdCommand::DeleteIndex(song.id));
                    app.queue = controller.fetch_queue();
                    app.update_filtered_indices();
                    app.set_notification("🗑 Removed track from Queue");
                }
            }
        }
        KeyCode::Char('c') if app.active_tab == ActiveTab::Playlist => {
            let _ = controller.execute(MpdCommand::ClearQueue);
            app.queue.clear();
            app.update_filtered_indices();
            app.set_notification("🗑 Cleared Queue");
        }

        _ => {}
    }
}
