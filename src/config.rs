use serde::{Deserialize, Serialize};
use std::{fs,path::PathBuf,time::Instant,};

use crate::config;

/// Configuration structure for the application
#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct Config {
    pub(crate) quit_key: char,
    pub(crate) Xommander: char,
    pub(crate) next_key: char,
    pub(crate) previous_key: char,
    pub(crate) play_pause_key: char,
    pub(crate) move_up_key: char,
    pub(crate) move_down_key: char,
    pub(crate) volup: char,
    pub(crate) voldown: char,
    pub(crate) selected_fg: String,
    pub(crate) selected_bg: String,
    pub(crate) unselected_fg: String,
    pub(crate) unselected_bg: String,
    pub(crate) top_fg: String,
    pub(crate) top_bg: String,
    pub(crate) bottom_fg: String,
    pub(crate) bottom_bg: String,
    pub(crate) notplaying_fg: String,
    pub(crate) notplaying_bg: String,
    pub(crate) rounding: bool,
    pub(crate) hide_controls: bool,
    pub(crate) startpage: String,
    pub(crate) refresh_interval: u64, //in ms
    pub(crate) change_page: char,
}

//We need the commander to pass to the main window, this should be left x, also add volume control and make the guide show the set values, add speaker selector per app and default, show the playing status
/// Default configuration values refresh is 300ms
impl Default for Config {
    fn default() -> Self {
        Config {
            quit_key: 'q',
            Xommander: 'x',
            next_key: 'n',
            previous_key: 'b',
            play_pause_key: 'm',
            move_up_key: 'c',
            move_down_key: 'v',
            volup: '+',
            voldown: '-',
            selected_fg: "White".to_string(),
            selected_bg: "Black".to_string(),
            unselected_fg: "Gray".to_string(),
            unselected_bg: "Black".to_string(),
            top_fg: "White".to_string(),
            top_bg: "Black".to_string(),
            bottom_fg: "Gray".to_string(),
            bottom_bg: "Black".to_string(),
            notplaying_fg: "White".to_string(),
            notplaying_bg: "Black".to_string(),
            rounding: true,
            hide_controls: false, //Accepts true and false
            startpage: "default".to_string(), //Supported are default, playback => same as default, sink => sink interface
            refresh_interval: 600,
            change_page: 'y',
        }
    }
}

///Load config file in a not scuffed way
pub fn load_config() -> Config {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("marstui/audio.toml");

    let config = if !config_path.exists() {
        // Config doesn't exist, write default
        let config = Config::default();
        let config_toml = toml::to_string(&config).unwrap();
        fs::create_dir_all(config_path.parent().unwrap()).expect("Failed to create config directory");
        fs::write(&config_path, config_toml).expect("Failed to write default config file");
        config
    } else {
        // Config exists, try to read & parse
        let config_content = fs::read_to_string(&config_path).unwrap_or_default();
        match toml::from_str::<Config>(&config_content) {
            Ok(cfg) => cfg, // Successfully loaded, return it
            Err(e) => {
                eprintln!("Warning: failed to parse config, backing up and using defaults: {}", e);

                // Backup the invalid file
                let backup_path = config_path.with_extension("backup");
                let _ = fs::copy(&config_path, &backup_path);

                // Return defaults
                Config::default()
            }
        }
    };
    config
}