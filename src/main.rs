use std::{thread, time::Duration};
use tui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block, Borders, Gauge, Paragraph}, Terminal
};
use crossterm::{
    event::{self, KeyCode}, execute, style::Print, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use zbus::Result;
use std::io::{self};
use mpris::{PlaybackStatus, Player, PlayerFinder};
use std::time::Instant;

mod config;
use config::load_config;
mod audio;
use audio::*;

fn main() -> Result<()> {
    // Load config
    let config = load_config();
    let refresh_ms = config.refresh_interval;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut audio = Terminal::new(backend)?;

    // Initialize PlayerFinder
    let finder = PlayerFinder::new().unwrap();
    let mut caches: Vec<Option<PlayerCache>> = Vec::new();
    let mut last_player_refresh = Instant::now();

    // Load variables
    let mut selected_index = 0;
    let mut scroll_offset = 0;
    let display_limit = 16;
    let volscopechange = 0.05;
    let mut page = config.startpage;
    let mut players = finder.find_all().unwrap_or_default();
    if caches.len() != players.len() {
        caches.resize_with(players.len(), || None);
    }
    if !players.is_empty() && selected_index >= players.len() {
        selected_index = players.len() - 1;
    }

    let mut eval = false;

    loop {
        // Refresh player list at most once per second
        if last_player_refresh.elapsed() >= Duration::from_secs(1) {
            players = finder.find_all().unwrap_or_default();
            if caches.len() != players.len() {
                caches.resize_with(players.len(), || None);
            }
            last_player_refresh = Instant::now();
        }

        // If no players, show "Not Playing"
        if players.is_empty() {
            let notplaying_fg = color_from_string(&config.notplaying_fg);
            let notplaying_bg = color_from_string(&config.notplaying_bg);

            audio.draw(|f| {
                let size = f.size();
                let nothingplayingblock = Paragraph::new("Not Playing")
                    .block(Block::default().borders(Borders::ALL).title("Nothing Playing"))
                    .style(Style::default().fg(notplaying_fg).bg(notplaying_bg));
                f.render_widget(nothingplayingblock, size);
            })?;

            // Quit key check even when nothing is playing
            if event::poll(Duration::from_millis(10))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char(config.quit_key) {
                        break;
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        let num_players = players.len();

        // Keep selected index within bounds
        if !players.is_empty() && selected_index >= num_players {
            selected_index = num_players - 1;
        }

        // Adjust scroll to keep the selected index visible
        if selected_index < scroll_offset {
            scroll_offset = selected_index;
        } else if selected_index >= scroll_offset + display_limit {
            scroll_offset = selected_index - display_limit + 1;
        }

        // Draw UI
        audio.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(4),     // Player list
                    Constraint::Length(3),  // Controls
                ].as_ref())
                .split(f.size());

            let top_fg = color_from_string(&config.top_fg);
            let top_bg = color_from_string(&config.top_bg);
            let bottom_fg = color_from_string(&config.bottom_fg);
            let bottom_bg = color_from_string(&config.bottom_bg);
            let selected_fg = color_from_string(&config.selected_fg);
            let selected_bg = color_from_string(&config.selected_bg);
            let unselected_fg = color_from_string(&config.unselected_fg);
            let unselected_bg = color_from_string(&config.unselected_bg);

            // Header for selected player
            if let Some(player) = players.get(selected_index) {
                if let Some(pl) = get_pl(player, &mut caches[selected_index], refresh_ms) {
                    let title = &pl.title;
                    let player_name = &pl.player_name;
                    let progress = pl.progress;
                    let mut volume = pl.volume;
                    if config.rounding {
                        volume = volume.round();
                    }

                    let header_text = format!(
                        "{} - ({}) - {:.0}% - V: {}%",
                        player_name, title, progress * 100.0, volume
                    );

                    let header = Paragraph::new(header_text)
                        .block(Block::default().borders(Borders::ALL).title("Currently Selected"))
                        .style(Style::default().fg(top_fg).bg(top_bg));

                    f.render_widget(header, chunks[0]);
                }
            }

            // Player list
            let player_gauges: Vec<Gauge> = players.iter().enumerate().map(|(i, player)| {
                if let Some(pl) = get_pl(player, &mut caches[i], refresh_ms) {
                    let title = &pl.title;
                    let app_name = &pl.player_name;
                    let progress = pl.progress;
                    let volume = pl.volume;
                    let playback = &pl.playback;

                    Gauge::default()
                        .block(Block::default().title(format!(
                            "{} - ({}) - V: {}% - {}",
                            title, app_name, volume.round(), playback
                        )))
                        .gauge_style(
                            Style::default()
                                .fg(if i == selected_index { selected_fg } else { unselected_fg })
                                .bg(if i == selected_index { selected_bg } else { unselected_bg })
                        )
                        .ratio(progress)
                } else {
                    Gauge::default()
                        .block(Block::default().title("Unknown or Unnamed song"))
                        .gauge_style(Style::default().fg(Color::White).bg(Color::Black))
                        .ratio(0.0)
                }
            }).collect();

            let visible_gauges = player_gauges.iter().skip(scroll_offset).take(display_limit);
            let gauge_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(2); display_limit])
                .split(chunks[1]);

            for (i, gauge) in visible_gauges.enumerate() {
                f.render_widget(gauge.clone(), gauge_layout[i]);
            }

            // Controls bar (only if hide_controls = false)
            if !config.hide_controls {
                let controls = Paragraph::new(format!(
                    "Controls: '{}': quit, '{}': next, '{}': previous, '{}': play/pause, '{}': up, '{}': down, vol up '{}', vol down '{}' change page '{}'",
                    config.quit_key, config.next_key, config.previous_key, config.play_pause_key,
                    config.move_up_key, config.move_down_key, config.volup, config.voldown, config.change_page,
                ))
                .block(Block::default().borders(Borders::ALL).title("Controls"))
                .style(Style::default().fg(bottom_fg).bg(bottom_bg));

                f.render_widget(controls, chunks[2]);
            }
        })?;

        // Handle key presses
        if event::poll(Duration::from_millis(10))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) if c == config.quit_key => break,
                    KeyCode::Char(c) if c == config.volup => set_vol(&players, volscopechange, selected_index),
                    KeyCode::Char(c) if c == config.voldown => set_vol(&players, -volscopechange, selected_index),
                    KeyCode::Char(c) if c == config.Xommander => break,
                    KeyCode::Char(c) if c == config.next_key => { players.get(selected_index).map(|p| p.next().ok()); },
                    KeyCode::Char(c) if c == config.previous_key => { players.get(selected_index).map(|p| p.previous().ok()); },
                    KeyCode::Char(c) if c == config.play_pause_key => {
                        if let Some(player) = players.get(selected_index) {
                            match player.get_playback_status() {
                                Ok(PlaybackStatus::Playing) => { player.pause().ok(); },
                                _ => { player.play().ok(); },
                            }
                        }
                    },
                    KeyCode::Char(c) if c == config.move_up_key => { if selected_index > 0 { selected_index -= 1; } },
                    KeyCode::Char(c) if c == config.move_down_key => { if selected_index < num_players - 1 { selected_index += 1; } },
                    KeyCode::Char(c) if c == config.change_page => { if page != "sink" { page = "sink".to_string(); } },
                    _ => {}
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    audio.show_cursor()?;
    Ok(())
}