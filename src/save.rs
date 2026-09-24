//! Save slots and settings, as JSON files in the user's data directory.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::game::Game;

/// Slot 0 is the autosave; 1-3 are the player's.
pub const SLOTS: usize = 4;

pub fn dir() -> PathBuf {
    let base = if let Ok(dir) = std::env::var("EMBERDEEP_SAVE_DIR") {
        PathBuf::from(dir)
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("Emberdeep")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/emberdeep")
    } else {
        PathBuf::from("saves")
    };
    let _ = fs::create_dir_all(&base);
    base
}

fn slot_path(slot: usize) -> PathBuf {
    dir().join(if slot == 0 { "autosave.json".to_string() } else { format!("slot{slot}.json") })
}

pub fn write(slot: usize, game: &Game) -> Result<(), String> {
    let json = serde_json::to_string(game).map_err(|e| e.to_string())?;
    let path = slot_path(slot);
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

pub fn read(slot: usize) -> Result<Game, String> {
    let text = fs::read_to_string(slot_path(slot)).map_err(|e| e.to_string())?;
    let mut game: Game = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    game.world.restore();
    Ok(game)
}

/// One line describing a slot, or None when it is empty.
pub fn describe(slot: usize) -> Option<String> {
    let game = read(slot).ok()?;
    let t = game.playtime as u64;
    let lead = &game.party[0];
    Some(format!(
        "{} · Lv {} · {} · {}:{:02}",
        game.place_name(),
        lead.level,
        game.difficulty.name(),
        t / 3600,
        (t / 60) % 60
    ))
}

pub fn any_save() -> Option<usize> {
    // The most recently written slot.
    (0..SLOTS)
        .filter_map(|s| fs::metadata(slot_path(s)).ok()?.modified().ok().map(|m| (m, s)))
        .max()
        .map(|(_, s)| s)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub music: f32,
    pub sfx: f32,
    /// Characters per second for dialogue.
    pub text_speed: f32,
    pub fullscreen: bool,
    pub battle_speed: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { music: 0.7, sfx: 0.8, text_speed: 55.0, fullscreen: false, battle_speed: 1.0 }
    }
}

pub fn load_settings() -> Settings {
    fs::read_to_string(dir().join("settings.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_settings(s: &Settings) {
    if let Ok(json) = serde_json::to_string_pretty(s) {
        let _ = fs::write(dir().join("settings.json"), json);
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Difficulty, Game};

    #[test]
    fn games_round_trip_through_json() {
        let mut game = Game::new(Difficulty::Normal, 42);
        game.enter_floor(3);
        game.set_flag("test_flag");
        let json = serde_json::to_string(&game).unwrap();
        let mut back: Game = serde_json::from_str(&json).unwrap();
        back.world.restore();
        assert_eq!(back.world.entities.len(), game.world.entities.len());
        assert!(back.flag("test_flag"));
        assert_eq!(back.deepest, 3);
    }
}
