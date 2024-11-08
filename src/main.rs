use std::{fs, path::PathBuf, thread, time::Duration};
use dirs::executable_dir;
use tui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Gauge, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color},
};
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Result};
use mpris::{PlaybackStatus, PlayerFinder, Player};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Configuration structure for the application
#[derive(Deserialize, Serialize, Debug)]
struct Config {
    quit_key: char,
    next_key: char,
    previous_key: char,
    play_pause_key: char,
    move_up_key: char,
    move_down_key: char,
    selected_fg: String,
    selected_bg: String,
    unselected_fg: String,
    unselected_bg: String,
    top_fg: String,
    top_bg: String,
    bottom_fg: String,
    bottom_bg: String,
}

/// Default configuration values
impl Default for Config {
    fn default() -> Self {
        Config {
            quit_key: 'q',
            next_key: 'n',
            previous_key: 'b',
            play_pause_key: 'm',
            move_up_key: 'c',
            move_down_key: 'v',
            selected_fg: "White".to_string(),
            selected_bg: "Black".to_string(),
            unselected_fg: "Gray".to_string(),
            unselected_bg: "Black".to_string(),
            top_fg: "White".to_string(),
            top_bg: "Black".to_string(),
            bottom_fg: "Gray".to_string(),
            bottom_bg: "Black".to_string(),
        }
    }
}

/// Load or create configuration file
fn load_config() -> Config {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("marstui/audio.toml");

    if !config_path.exists() {
        let default_config = Config::default();
        let config_toml = toml::to_string(&default_config).unwrap();
        fs::create_dir_all(config_path.parent().unwrap()).expect("Failed to create config directory");
        fs::write(&config_path, config_toml).expect("Failed to write default config file");
        default_config
    } else {
        //lets take a backup and kill the file
        let junk = Command::new("mv .config/marstui/audio.toml .config/marstui/audio.backup").arg("-l").output();//.expect("Nothing Like my life");
        let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");
        toml::from_str(&config_content).expect("Failed to parse config file")
    }
}

/// Get song information, including title, player name, and progress percentage.
fn get_song_info(player: &Player) -> Option<(String, String, f64)> {
    if let Ok(metadata) = player.get_metadata() {
        let title = metadata.title().unwrap_or("Unkown or Unamed Title").to_string();
        let player_name = player.identity().to_string();
        let length = metadata.length().map_or(0.0, |l| l.as_secs_f64());
        let position = player.get_position().map_or(0.0, |p| p.as_secs_f64());
        let progress = if length > 0.0 { position / length } else { 0.0 };
        Some((title, player_name, progress))
    } else {
        None
    }
}

fn color_from_string(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "grey" => Color::Gray,
        "gray" => Color::Gray,
        _ => Color::Reset,  // Default to Reset if the color is not recognized
    }
}


fn main() -> Result<()> {
    // Load config
    let config = load_config();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut audio = Terminal::new(backend)?;

    // Initialize PlayerFinder
    let finder = PlayerFinder::new().unwrap();

    // Track selected player and scroll offset
    let mut selected_index = 0;
    let mut scroll_offset = 0;
    let display_limit = 5; // Number of players to display at once

    loop {
        // Refresh the list of active players
        let players: Vec<Player> = finder.find_all().unwrap_or_else(|_| vec![]);
        let num_players = players.len();

        // Check if there are active players
        if num_players == 0 {
            eprintln!("No active media players found!");
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen)?;
            return Ok(());
        }

        // Keep selected index within bounds
        if selected_index >= num_players {
            selected_index = num_players - 1;
        }

        // Adjust scroll to keep the selected index visible within the display limit
        if selected_index < scroll_offset {
            scroll_offset = selected_index;
        } else if selected_index >= scroll_offset + display_limit {
            scroll_offset = selected_index - display_limit + 1;
        }

        // Render TUI
        audio.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),  // Header
                        Constraint::Min(4),    // Player list
                        Constraint::Length(3),  // Controls
                    ].as_ref()
                )
                .split(f.size());

                let top_fg = color_from_string(&config.top_fg);
                let top_bg = color_from_string(&config.top_bg);
                let bottom_fg = color_from_string(&config.bottom_fg);
                let bottom_bg = color_from_string(&config.bottom_bg);  

            // Display the selected player's info in the header
            if let Some(player) = players.get(selected_index) {
                if let Some((title, player_name, progress)) = get_song_info(player) {
                    let header_text = format!("{} - ({}) - {:.0}%", player_name, title, progress * 100.0);
                    let header = Paragraph::new(header_text)
                        .block(Block::default().borders(Borders::ALL).title("Currently Selected"))
                        .style(Style::default().fg(top_fg).bg(top_bg));
                    f.render_widget(header, chunks[0]);
                }
            }

            let selected_fg = color_from_string(&config.selected_fg);
            let selected_bg = color_from_string(&config.selected_bg);
            let unselected_fg = color_from_string(&config.unselected_fg);
            let unselected_bg = color_from_string(&config.unselected_bg);             

            // Render each player as a Gauge
            let player_gauges: Vec<Gauge> = players.iter().enumerate().map(|(i, player)| {
                if let Some((title, app_name, progress)) = get_song_info(player) {
                    Gauge::default()
                        .block(Block::default().title(format!("{} - ({})", title, app_name)))
                        .gauge_style(Style::default().fg(if i == selected_index { selected_fg } else { unselected_fg }).bg(if i == selected_index { selected_bg } else { unselected_bg }))
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

            // Render control instructions
            let controls = Paragraph::new(format!(
                "Controls: '{}': quit, '{}': next, '{}': previous, '{}': play/pause, '{}': up, '{}': down",
                config.quit_key, config.next_key, config.previous_key, config.play_pause_key, config.move_up_key, config.move_down_key,
            ))
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(bottom_fg).bg(bottom_bg));

            f.render_widget(controls, chunks[2]);
        })?;

        // Handle key presses based on config
        if event::poll(Duration::from_millis(10))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) if c == config.quit_key => break,
                    KeyCode::Char(c) if c == config.next_key => {
                        if let Some(player) = players.get(selected_index) {
                            player.next().ok();
                        }
                    }
                    KeyCode::Char(c) if c == config.previous_key => {
                        if let Some(player) = players.get(selected_index) {
                            player.previous().ok();
                        }
                    }
                    KeyCode::Char(c) if c == config.play_pause_key => {
                        if let Some(player) = players.get(selected_index) {
                            match player.get_playback_status() {
                                Ok(PlaybackStatus::Playing) => { player.pause().ok(); },
                                _ => { player.play().ok(); },
                            }                            
                        }
                    }
                    KeyCode::Char(c) if c == config.move_up_key => {
                        if selected_index > 0 { selected_index -= 1; }
                    }
                    KeyCode::Char(c) if c == config.move_down_key => {
                        if selected_index < num_players - 1 { selected_index += 1; }
                    }
                    _ => {}
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    audio.show_cursor()?;
    Ok(())
}
