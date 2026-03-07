use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rocket::{RocketDimensions, RocketFlightParameters};

pub const STARTING_BALANCE: f64 = 50.0;

#[derive(Serialize, Deserialize, Clone)]
pub struct RocketSave {
    pub name: String,
    pub dimensions: RocketDimensions,
    pub flight_params: RocketFlightParameters,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerMeta {
    pub name: String,
    #[serde(default = "default_balance")]
    pub balance: f64,
}

fn default_balance() -> f64 {
    STARTING_BALANCE
}

#[derive(Resource)]
pub struct PlayerBalance(pub f64);

impl Default for PlayerBalance {
    fn default() -> Self {
        Self(STARTING_BALANCE)
    }
}

#[derive(Resource, Default)]
pub struct SaveState {
    pub player_name: Option<String>,
    pub rocket_saves: Vec<String>,
    pub rocket_name_buf: String,
    pub player_name_buf: String,
    pub status_message: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
mod io {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    pub fn players_dir() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("rocket-lab").join("players")
    }

    fn player_dir(player: &str) -> PathBuf {
        players_dir().join(sanitize_name(player))
    }

    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn rocket_filename(rocket_name: &str) -> String {
        format!("{}.json", sanitize_name(rocket_name))
    }

    pub fn list_players() -> Vec<String> {
        let dir = players_dir();
        let Ok(entries) = fs::read_dir(&dir) else {
            return Vec::new();
        };
        let mut names = Vec::new();
        for entry in entries.flatten() {
            if entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
                && entry.path().join("player.json").exists()
            {
                names.push(name.to_string());
            }
        }
        names.sort();
        names
    }

    pub fn list_rockets(player: &str) -> Vec<String> {
        let dir = player_dir(player);
        let Ok(entries) = fs::read_dir(&dir) else {
            return Vec::new();
        };
        let mut names = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && stem != "player"
            {
                names.push(stem.to_string());
            }
        }
        names.sort();
        names
    }

    pub fn ensure_player_dir(player: &str) -> Result<(), String> {
        let dir = player_dir(player);
        let meta_path = dir.join("player.json");
        if meta_path.exists() {
            let existing = fs::read_to_string(&meta_path)
                .map_err(|e| format!("Failed to read player.json: {e}"))?;
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&existing)
                && let Some(existing_name) = meta.get("name").and_then(|v| v.as_str())
                && existing_name != player
            {
                return Err(format!(
                    "Name conflicts with existing player \"{existing_name}\""
                ));
            }
        }
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create player dir: {e}"))?;
        if !meta_path.exists() {
            let meta = PlayerMeta {
                name: player.to_string(),
                balance: STARTING_BALANCE,
            };
            let json = serde_json::to_string_pretty(&meta)
                .map_err(|e| format!("Failed to serialize player meta: {e}"))?;
            fs::write(&meta_path, json).map_err(|e| format!("Failed to write player.json: {e}"))?;
        }
        Ok(())
    }

    pub fn load_player_meta(player: &str) -> Result<PlayerMeta, String> {
        let path = player_dir(player).join("player.json");
        let json =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read player.json: {e}"))?;
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse player.json: {e}"))
    }

    pub fn save_player_meta(meta: &PlayerMeta) -> Result<(), String> {
        ensure_player_dir(&meta.name)?;
        let path = player_dir(&meta.name).join("player.json");
        let json = serde_json::to_string_pretty(meta)
            .map_err(|e| format!("Failed to serialize player meta: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write player.json: {e}"))
    }

    pub fn save_rocket(
        player: &str,
        rocket_name: &str,
        dims: &RocketDimensions,
        params: &RocketFlightParameters,
    ) -> Result<(), String> {
        ensure_player_dir(player)?;
        let path = player_dir(player).join(rocket_filename(rocket_name));
        if path.exists()
            && let Ok(existing_json) = fs::read_to_string(&path)
            && let Ok(existing) = serde_json::from_str::<RocketSave>(&existing_json)
            && existing.name != rocket_name
        {
            return Err(format!(
                "Name conflicts with existing rocket \"{}\"",
                existing.name
            ));
        }
        let save = RocketSave {
            name: rocket_name.to_string(),
            dimensions: dims.clone(),
            flight_params: params.clone(),
        };
        let json = serde_json::to_string_pretty(&save)
            .map_err(|e| format!("Failed to serialize rocket: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write rocket file: {e}"))?;
        Ok(())
    }

    pub fn load_rocket(player: &str, rocket_name: &str) -> Result<RocketSave, String> {
        let path = player_dir(player).join(rocket_filename(rocket_name));
        let json =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read rocket file: {e}"))?;
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse rocket file: {e}"))
    }

    pub fn delete_rocket(player: &str, rocket_name: &str) -> Result<(), String> {
        let path = player_dir(player).join(rocket_filename(rocket_name));
        fs::remove_file(&path).map_err(|e| format!("Failed to delete rocket file: {e}"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use io::*;
