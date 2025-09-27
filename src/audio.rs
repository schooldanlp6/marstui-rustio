//use crossterm::style::Color;
use tui::style::Color;
use std::time::Instant;
use mpris::Player;


#[derive(Clone)]
pub struct PlayerCache {
    pub(crate) title: String,
    pub(crate) player_name: String,
    pub(crate) progress: f64,
    pub(crate) volume: f64,
    pub(crate) playback: String,
    pub(crate) last_updated: Instant,
}

pub fn drawplayer(){

}

//Get Players from mpris
pub fn get_pl(player: &Player, cache: &mut Option<PlayerCache>, refresh_ms: u64) -> Option<PlayerCache> {
    let now = Instant::now();
    if let Some(pl) = cache {
        if now.duration_since(pl.last_updated).as_millis() < refresh_ms as u128 {
            return cache.clone(); // skip DBus query
        }
    }

    if let Ok(metadata) = player.get_metadata() {
        let title = metadata.title().unwrap_or("Unknown Title").to_string();
        let player_name = player.identity().to_string();
        let length = metadata.length().map_or(0.0, |l| l.as_secs_f64());
        let position = player.get_position().map_or(0.0, |p| p.as_secs_f64());
        let progress = if length > 0.0 { position / length } else { 0.0 };
        let volume = player.get_volume().unwrap_or(0.22) * 100.0;
        let playback = "not implemented".to_string();

        let new_cache = PlayerCache {
            title,
            player_name,
            progress,
            volume,
            playback,
            last_updated: now,
        };
        *cache = Some(new_cache);
    }
    cache.clone()
}


pub fn set_vol(players: &[Player], change: f64, selected_index: usize) {
    if let Some(player) = players.get(selected_index) {
        // Get the current volume from the player
        let volnow = match player.get_volume() {Ok(volume) => volume,Err(_) => 0.22,};
        let volnew = (volnow + change).clamp(0.0, 1.0);
        let mut failed;
        if let Err(newvol) = player.set_volume(volnew){failed = newvol;}else{let failed = "notfailed";}
    }
}



// added undocumented color schem nc means no color
pub fn color_from_string(color_str: &str) -> Color {
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
        "nc" => Color::Reset,
        _ => Color::Reset,  // Default to Reset if the color is not recognized
    }
}