use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const WALLPAPER_COUNT: usize = 3;
const ACCENT_COUNT: usize = 5;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Preferences {
    pub wallpaper: usize,
    pub accent: usize,
    pub scale: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub anchor: i32,
    pub volume: f32,
    pub muted: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            wallpaper: 0,
            accent: 0,
            scale: 1.0,
            position_x: 0.5,
            position_y: 0.5,
            anchor: 4,
            volume: 0.35,
            muted: false,
        }
    }
}

impl Preferences {
    pub fn normalized(mut self) -> Self {
        self.wallpaper = self.wallpaper.min(WALLPAPER_COUNT - 1);
        self.accent = self.accent.min(ACCENT_COUNT - 1);
        self.scale = if self.scale.is_finite() {
            self.scale.clamp(0.7, 1.1)
        } else {
            1.0
        };
        if !self.position_x.is_finite() {
            self.position_x = 0.5;
        }
        if !self.position_y.is_finite() {
            self.position_y = 0.5;
        }
        self.position_x = self.position_x.clamp(0.0, 1.0);
        self.position_y = self.position_y.clamp(0.0, 1.0);
        if !(-1..=8).contains(&self.anchor) {
            self.anchor = -1;
        }
        self.volume = if self.volume.is_finite() {
            self.volume.clamp(0.0, 1.0)
        } else {
            0.35
        };
        self
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Não foi possível salvar configurações em '{path}': {source}")]
    Save {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Configurações em '{path}' não puderam ser preparadas: {source}")]
    Prepare {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Configurações em '{path}' não puderam ser codificadas: {source}")]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub fn preferences_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("AppData/Roaming"))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("GTA VI Countdown").join("settings.json")
}

pub fn load_preferences(path: &Path) -> Preferences {
    let Ok(contents) = fs::read(path) else {
        return Preferences::default();
    };
    serde_json::from_slice::<Preferences>(&contents)
        .unwrap_or_default()
        .normalized()
}

pub fn save_preferences(path: &Path, preferences: &Preferences) -> Result<(), SettingsError> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|source| SettingsError::Prepare {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut temporary = tempfile::Builder::new()
        .suffix(".json")
        .tempfile_in(parent)
        .map_err(|source| SettingsError::Prepare {
            path: parent.to_path_buf(),
            source,
        })?;
    let bytes = serde_json::to_vec_pretty(&preferences.clone().normalized()).map_err(|source| {
        SettingsError::Encode {
            path: path.to_path_buf(),
            source,
        }
    })?;
    temporary
        .write_all(&bytes)
        .map_err(|source| SettingsError::Save {
            path: path.to_path_buf(),
            source,
        })?;
    temporary
        .persist(path)
        .map_err(|error| SettingsError::Save {
            path: path.to_path_buf(),
            source: error.error,
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Preferences, load_preferences, save_preferences};

    #[test]
    fn defaults_are_used_for_missing_or_malformed_settings() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        assert_eq!(load_preferences(&path), Preferences::default());
        std::fs::write(&path, b"{broken").unwrap();
        assert_eq!(load_preferences(&path), Preferences::default());
    }

    #[test]
    fn settings_round_trip_and_normalize_out_of_range_values() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        let preferences = Preferences {
            wallpaper: 50,
            accent: 50,
            scale: 3.0,
            position_x: -2.0,
            position_y: 2.0,
            anchor: 12,
            volume: 3.0,
            muted: true,
        };
        save_preferences(&path, &preferences).unwrap();
        assert_eq!(
            load_preferences(&path),
            Preferences {
                wallpaper: 2,
                accent: 4,
                scale: 1.1,
                position_x: 0.0,
                position_y: 1.0,
                anchor: -1,
                volume: 1.0,
                muted: true,
            }
        );
        let updated = Preferences {
            scale: 0.85,
            muted: false,
            ..Preferences::default()
        };
        save_preferences(&path, &updated).unwrap();
        assert_eq!(load_preferences(&path), updated);
    }
}
