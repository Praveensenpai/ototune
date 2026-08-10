use crate::app::{ActiveTab, AppState};
use crate::mpd_client::PlaybackState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};

pub fn render_ui(frame: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Header / Player Status
            Constraint::Min(10),   // Main Dual-Pane Body
            Constraint::Length(2), // Footer / Keybind Hints
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_body(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    if app.show_help {
        render_help_modal(frame);
    }
}

fn render_header(frame: &mut Frame, app: &AppState, area: Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(255, 182, 193)))
        .title(Span::styled(
            " 🌸 ototune MPD Player ",
            Style::default()
                .fg(Color::Rgb(255, 192, 203))
                .add_modifier(Modifier::BOLD),
        ));

    let inner = header_block.inner(area);
    frame.render_widget(header_block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title & Artist
            Constraint::Length(1), // Album & Playback Badges
            Constraint::Length(1), // Progress Gauge
            Constraint::Length(1), // Notification / Info Status
        ])
        .split(inner);

    let (song_title, song_artist, song_album) = if let Some(song) = &app.status.current_song {
        (song.title.as_str(), song.artist.as_str(), song.album.as_str())
    } else {
        ("No track playing", "Unknown Artist", "Unknown Album")
    };

    let title_spans = vec![
        Span::styled("🎵 ", Style::default().fg(Color::LightMagenta)),
        Span::styled(
            song_title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  —  ", Style::default().fg(Color::DarkGray)),
        Span::styled(song_artist, Style::default().fg(Color::LightCyan)),
    ];
    frame.render_widget(Paragraph::new(Line::from(title_spans)), layout[0]);

    let state_badge = match app.status.state {
        PlaybackState::Playing => Span::styled(" ▶ PLAYING ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        PlaybackState::Paused => Span::styled(" ⏸ PAUSED ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
        PlaybackState::Stopped => Span::styled(" ⏹ STOPPED ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        PlaybackState::Disconnected => Span::styled(" ❌ OFFLINE ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
    };

    let repeat_badge = if app.status.repeat {
        Span::styled(" [Repeat: ON] ", Style::default().fg(Color::LightGreen))
    } else {
        Span::styled(" [Repeat: OFF] ", Style::default().fg(Color::DarkGray))
    };

    let random_badge = if app.status.random {
        Span::styled("[Random: ON] ", Style::default().fg(Color::LightGreen))
    } else {
        Span::styled("[Random: OFF] ", Style::default().fg(Color::DarkGray))
    };

    let single_badge = if app.status.single {
        Span::styled("[Single: ON] ", Style::default().fg(Color::LightMagenta))
    } else {
        Span::styled("[Single: OFF] ", Style::default().fg(Color::DarkGray))
    };

    let resume_badge = if app.resume_mode {
        Span::styled("[Resume: ON] ", Style::default().fg(Color::LightYellow))
    } else {
        Span::styled("[Resume: OFF] ", Style::default().fg(Color::DarkGray))
    };

    let vol_text = if app.status.volume >= 0 {
        format!("🔊 {}%", app.status.volume)
    } else {
        "🔊 N/A".to_string()
    };

    let row2_spans = vec![
        state_badge,
        Span::raw("  📁 "),
        Span::styled(song_album, Style::default().fg(Color::Gray)),
        Span::raw("  │ "),
        repeat_badge,
        random_badge,
        single_badge,
        resume_badge,
        Span::raw("│ "),
        Span::styled(vol_text, Style::default().fg(Color::LightYellow)),
    ];
    frame.render_widget(Paragraph::new(Line::from(row2_spans)), layout[1]);

    let elapsed_sec = app.status.elapsed.map(|d| d.as_secs()).unwrap_or(0);
    let total_sec = app.status.total.map(|d| d.as_secs()).unwrap_or(0);
    let ratio = if total_sec > 0 {
        (elapsed_sec as f64 / total_sec as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let label = format!(
        "{:02}:{:02} / {:02}:{:02}",
        elapsed_sec / 60,
        elapsed_sec % 60,
        total_sec / 60,
        total_sec % 60
    );

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Rgb(255, 105, 180)).bg(Color::Rgb(40, 40, 50)))
        .ratio(ratio)
        .label(label);
    frame.render_widget(gauge, layout[2]);

    if let Some(notif) = app.get_active_notification() {
        let toast = Paragraph::new(format!("✨ {}", notif))
            .style(Style::default().fg(Color::LightYellow).add_modifier(Modifier::ITALIC));
        frame.render_widget(toast, layout[3]);
    }
}

fn render_body(frame: &mut Frame, app: &AppState, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Left: Library Browser
            Constraint::Percentage(65), // Right: Active Queue / Playlist
        ])
        .split(area);

    render_browser_pane(frame, app, body_chunks[0]);
    render_playlist_pane(frame, app, body_chunks[1]);
}

fn render_browser_pane(frame: &mut Frame, app: &AppState, area: Rect) {
    let is_active = app.active_tab == ActiveTab::Browser;
    let border_color = if is_active {
        Color::Rgb(255, 182, 193)
    } else {
        Color::DarkGray
    };

    let path_display = if app.browser_path.is_empty() {
        "Root".to_string()
    } else {
        app.browser_path.clone()
    };

    let title_text = if app.show_hidden {
        format!(" 📂 Library [{}] (All) ", path_display)
    } else {
        format!(" 📂 Library [{}] ", path_display)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_active { BorderType::Double } else { BorderType::Rounded })
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            title_text,
            Style::default().fg(if is_active { Color::LightCyan } else { Color::Gray }),
        ));

    let items: Vec<ListItem> = app
        .browser_entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = is_active && idx == app.browser_selected;
            let icon = if entry.is_dir { "📁 " } else { "🎵 " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 182, 193))
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::LightCyan)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▸ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(icon, style),
                Span::styled(&entry.name, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_playlist_pane(frame: &mut Frame, app: &AppState, area: Rect) {
    let is_active = app.active_tab == ActiveTab::Playlist;
    let border_color = if is_active {
        Color::Rgb(255, 182, 193)
    } else {
        Color::DarkGray
    };

    let search_status = if app.search_mode {
        format!(" 🔍 SEARCH: {}█ ", app.search_query)
    } else if !app.search_query.is_empty() {
        format!(" 🔍 Filtered: '{}' ", app.search_query)
    } else {
        format!(" 🎵 Queue ({}) ", app.queue.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_active { BorderType::Double } else { BorderType::Rounded })
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            search_status,
            Style::default().fg(if is_active { Color::LightMagenta } else { Color::Gray }),
        ));

    let visible_indices = if app.search_mode || !app.search_query.is_empty() {
        &app.filtered_queue_indices[..]
    } else {
        &[]
    };

    let items_count = if app.search_mode || !app.search_query.is_empty() {
        visible_indices.len()
    } else {
        app.queue.len()
    };

    let current_pos = app.status.current_song.as_ref().map(|s| s.pos);

    let items: Vec<ListItem> = (0..items_count)
        .map(|idx| {
            let actual_song_idx = if app.search_mode || !app.search_query.is_empty() {
                visible_indices[idx]
            } else {
                idx
            };

            let song = &app.queue[actual_song_idx];
            let is_selected = is_active && idx == app.playlist_selected;
            let is_playing = current_pos == Some(song.pos);

            let cursor = if is_playing && is_selected {
                "▶▸"
            } else if is_playing {
                "▶ "
            } else if is_selected {
                "▸ "
            } else {
                "  "
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 192, 203))
                    .add_modifier(Modifier::BOLD)
            } else if is_playing {
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let dur_str = song
                .duration
                .map(|d| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60))
                .unwrap_or_else(|| "--:--".into());

            let line_spans = vec![
                Span::styled(format!("{:2} ", cursor), style),
                Span::styled(format!("{:3}. ", song.pos + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} ", song.title), style),
                Span::styled(format!("— {} ", song.artist), Style::default().fg(Color::Gray)),
                Span::styled(format!("({})", dur_str), Style::default().fg(Color::DarkGray)),
            ];

            ListItem::new(Line::from(line_spans))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame, _app: &AppState, area: Rect) {
    let keybinds = vec![
        Span::styled(" [Tab] ", Style::default().fg(Color::LightYellow)),
        Span::raw("Switch Pane  "),
        Span::styled(" [Space] ", Style::default().fg(Color::LightYellow)),
        Span::raw("Play/Pause  "),
        Span::styled(" [i] ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
        Span::raw("Immersion  "),
        Span::styled(" [m] ", Style::default().fg(Color::LightYellow)),
        Span::raw("Resume  "),
        Span::styled(" [r] ", Style::default().fg(Color::LightGreen)),
        Span::raw("Random  "),
        Span::styled(" [e] ", Style::default().fg(Color::LightGreen)),
        Span::raw("Repeat  "),
        Span::styled(" [l] ", Style::default().fg(Color::LightCyan)),
        Span::raw("Single  "),
        Span::styled(" [.] ", Style::default().fg(Color::LightCyan)),
        Span::raw("Hidden  "),
        Span::styled(" [+/-] ", Style::default().fg(Color::LightYellow)),
        Span::raw("Vol  "),
        Span::styled(" [/] ", Style::default().fg(Color::LightYellow)),
        Span::raw("Search  "),
        Span::styled(" [?] ", Style::default().fg(Color::White)),
        Span::raw("Help  "),
        Span::styled(" [q] ", Style::default().fg(Color::LightRed)),
        Span::raw("Quit"),
    ];

    let paragraph = Paragraph::new(Line::from(keybinds))
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    frame.render_widget(paragraph, area);
}

fn render_help_modal(frame: &mut Frame) {
    let area = frame.area();
    let popup_area = Rect {
        x: area.width.saturating_sub(62) / 2,
        y: area.height.saturating_sub(24) / 2,
        width: 62.min(area.width),
        height: 24.min(area.height),
    };

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "🌸 ototune Keybindings Cheatsheet",
            Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Tab            ", Style::default().fg(Color::LightYellow)),
            Span::raw("Toggle focus between Library & Playlist"),
        ]),
        Line::from(vec![
            Span::styled(" Space          ", Style::default().fg(Color::LightYellow)),
            Span::raw("Toggle Play / Pause"),
        ]),
        Line::from(vec![
            Span::styled(" m              ", Style::default().fg(Color::LightYellow)),
            Span::raw("Toggle Resume Mode (Remember exact playback position)"),
        ]),
        Line::from(vec![
            Span::styled(" i              ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
            Span::raw("Context Immersion Mode (Folder or Root)"),
        ]),
        Line::from(vec![
            Span::styled(" I (Shift+I)    ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
            Span::raw("Global Full Library Immersion Mode"),
        ]),
        Line::from(vec![
            Span::styled(" r              ", Style::default().fg(Color::LightGreen)),
            Span::raw("Toggle Random Playback (ON / OFF)"),
        ]),
        Line::from(vec![
            Span::styled(" e              ", Style::default().fg(Color::LightGreen)),
            Span::raw("Toggle Repeat Mode (ON / OFF)"),
        ]),
        Line::from(vec![
            Span::styled(" l              ", Style::default().fg(Color::LightCyan)),
            Span::raw("Toggle Single Track Loop (ON / OFF)"),
        ]),
        Line::from(vec![
            Span::styled(" .              ", Style::default().fg(Color::LightCyan)),
            Span::raw("Toggle Showing Hidden (.stfolder) Files"),
        ]),
        Line::from(vec![
            Span::styled(" Enter          ", Style::default().fg(Color::LightGreen)),
            Span::raw("Play queued track / Open dir / Queue track"),
        ]),
        Line::from(vec![
            Span::styled(" d / Delete     ", Style::default().fg(Color::LightRed)),
            Span::raw("Remove track from Queue"),
        ]),
        Line::from(vec![
            Span::styled(" c              ", Style::default().fg(Color::LightRed)),
            Span::raw("Clear Queue"),
        ]),
        Line::from(vec![
            Span::styled(" + / -          ", Style::default().fg(Color::LightYellow)),
            Span::raw("Volume Up / Down by 5%"),
        ]),
        Line::from(vec![
            Span::styled(" /              ", Style::default().fg(Color::LightYellow)),
            Span::raw("Start Live Playlist Search"),
        ]),
        Line::from(vec![
            Span::styled(" q              ", Style::default().fg(Color::LightRed)),
            Span::raw("Quit ototune"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or ? to close this overlay",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(255, 182, 193)))
        .style(Style::default().bg(Color::Rgb(25, 25, 35)));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup_area);
}
