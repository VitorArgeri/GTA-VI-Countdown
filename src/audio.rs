use std::{
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use thiserror::Error;

const DEFAULT_VOLUME: f32 = 0.35;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Arquivo de áudio ausente em '{0}'; esperado um WAV válido em assets/audio.")]
    Missing(PathBuf),
    #[error("Arquivo de áudio inválido em '{path}': {reason}")]
    Invalid { path: PathBuf, reason: String },
    #[error("Saída de áudio indisponível: {0}")]
    Output(String),
}

pub struct AudioPlayback {
    path: PathBuf,
    _output: rodio::MixerDeviceSink,
    player: Player,
    volume: f32,
    muted: bool,
    started: bool,
}

impl AudioPlayback {
    pub fn open(path: PathBuf) -> Result<Self, AudioError> {
        validate_track(&path)?;
        let output = DeviceSinkBuilder::open_default_sink()
            .map_err(|error| AudioError::Output(error.to_string()))?;
        let player = Player::connect_new(output.mixer());
        player.set_volume(DEFAULT_VOLUME);
        player.pause();
        Ok(Self {
            path,
            _output: output,
            player,
            volume: DEFAULT_VOLUME,
            muted: false,
            started: false,
        })
    }

    pub fn start_loop(&mut self) -> Result<(), AudioError> {
        if self.started {
            self.player.play();
            return Ok(());
        }
        let file = File::open(&self.path).map_err(|error| AudioError::Invalid {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;
        let decoder = Decoder::try_from(file).map_err(|error| AudioError::Invalid {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;
        self.player.append(decoder.repeat_infinite());
        self.player.play();
        self.started = true;
        Ok(())
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.apply_volume();
        self.muted
    }

    pub fn change_volume(&mut self, delta: f32) -> f32 {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.apply_volume();
        self.volume
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    fn apply_volume(&self) {
        self.player
            .set_volume(if self.muted { 0.0 } else { self.volume });
    }
}

pub fn validate_track(path: &Path) -> Result<Duration, AudioError> {
    if !path.is_file() {
        return Err(AudioError::Missing(path.to_path_buf()));
    }
    let file = File::open(path).map_err(|error| AudioError::Invalid {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    let decoder = Decoder::try_from(file).map_err(|error| AudioError::Invalid {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    valid_duration(path, decoder.total_duration())
}

fn valid_duration(path: &Path, duration: Option<Duration>) -> Result<Duration, AudioError> {
    duration
        .filter(|value| !value.is_zero())
        .ok_or_else(|| AudioError::Invalid {
            path: path.to_path_buf(),
            reason: "esperado áudio WAV com duração maior que zero".to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::{AudioError, valid_duration, validate_track};
    use std::{path::PathBuf, time::Duration};

    #[test]
    fn reports_missing_audio_with_its_path() {
        let path = PathBuf::from("missing-love-is-a-long-road.wav");
        assert!(matches!(validate_track(&path), Err(AudioError::Missing(_))));
    }

    #[test]
    fn rejects_malformed_audio_files() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("invalid.wav");
        std::fs::write(&path, b"not a wav file").unwrap();
        assert!(matches!(
            validate_track(&path),
            Err(AudioError::Invalid { .. })
        ));
    }

    #[test]
    fn rejects_zero_length_audio() {
        let path = PathBuf::from("empty.wav");
        assert!(matches!(
            valid_duration(&path, Some(Duration::ZERO)),
            Err(AudioError::Invalid { .. })
        ));
    }

    #[test]
    fn accepts_a_nonempty_audio_duration() {
        let path = PathBuf::from("track.wav");
        assert_eq!(
            valid_duration(&path, Some(Duration::from_secs(1))).unwrap(),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn validates_the_bundled_launch_track() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/audio/love-is-a-long-road.wav");
        let duration = validate_track(&path).expect("the extracted launch song is valid");
        assert!(duration > Duration::from_secs(30));
    }
}
