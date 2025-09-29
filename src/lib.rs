mod audio;
mod config;
mod ui;
use audio::{PlayerCache, color_from_string, get_pl, set_vol};
use config::Config as audioConfig;
use config::load_config;