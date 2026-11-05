use std::{
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::Duration,
};

use hound::{SampleFormat, WavSpec, WavWriter};
use symphonia::core::{
    audio::{AudioBufferRef, SampleBuffer, SignalSpec},
    codecs::{Decoder, DecoderOptions},
    errors::Error as SymphoniaError,
    formats::{FormatOptions, FormatReader, Packet},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("Vídeo de origem ausente em '{0}'.")]
    MissingInput(PathBuf),
    #[error("Não foi possível ler '{path}': {source}")]
    ReadInput {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("O arquivo '{0}' não contém uma faixa de áudio reconhecida.")]
    NoAudio(PathBuf),
    #[error("Não foi possível decodificar o áudio de '{path}': {reason}")]
    Decode { path: PathBuf, reason: String },
    #[error("Falha ao gravar WAV: {0}")]
    Wav(#[from] hound::Error),
    #[error("Falha ao gravar arquivo temporário: {0}")]
    Temporary(#[from] std::io::Error),
}

struct AudioTrack {
    id: u32,
    decoder: Box<dyn Decoder>,
}

pub fn extract_to_wav(input: &Path, output: &Path) -> Result<Duration, ExtractError> {
    if !input.is_file() {
        return Err(ExtractError::MissingInput(input.to_path_buf()));
    }
    let file = File::open(input).map_err(|source| ExtractError::ReadInput {
        path: input.to_path_buf(),
        source,
    })?;
    let mut format = probe_media(file, input)?;
    let track = create_audio_track(format.as_mut(), input)?;
    write_decoded_audio(format.as_mut(), track, input, output)
}

fn probe_media(file: File, input: &Path) -> Result<Box<dyn FormatReader>, ExtractError> {
    let hint = media_hint(input);
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let result = symphonia::default::get_probe().format(
        &hint,
        stream,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    );
    result
        .map(|probed| probed.format)
        .map_err(|error| decode_error(input, error))
}

fn media_hint(input: &Path) -> Hint {
    let mut hint = Hint::new();
    if let Some(extension) = input.extension().and_then(|value| value.to_str()) {
        hint.with_extension(extension);
    }
    hint
}

fn create_audio_track(
    format: &mut dyn FormatReader,
    input: &Path,
) -> Result<AudioTrack, ExtractError> {
    let track = format
        .tracks()
        .iter()
        .find(|track| {
            track.codec_params.sample_rate.is_some() || track.codec_params.channels.is_some()
        })
        .ok_or_else(|| ExtractError::NoAudio(input.to_path_buf()))?;
    let params = track.codec_params.clone();
    let decoder = symphonia::default::get_codecs()
        .make(&params, &DecoderOptions::default())
        .map_err(|error| decode_error(input, format!("codec {:?}: {error}", params.codec)))?;
    Ok(AudioTrack {
        id: track.id,
        decoder,
    })
}

fn wave_spec(spec: &SignalSpec) -> WavSpec {
    WavSpec {
        channels: spec.channels.count() as u16,
        sample_rate: spec.rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    }
}

fn write_decoded_audio(
    format: &mut dyn FormatReader,
    mut track: AudioTrack,
    input: &Path,
    output: &Path,
) -> Result<Duration, ExtractError> {
    let first_packet = next_track_packet(format, track.id, input)?
        .ok_or_else(|| ExtractError::NoAudio(input.to_path_buf()))?;
    let first_buffer = track
        .decoder
        .decode(&first_packet)
        .map_err(|error| decode_error(input, error))?;
    let spec = wave_spec(first_buffer.spec());
    let folder = output.parent().unwrap_or(Path::new("."));
    let mut temporary = tempfile::Builder::new()
        .suffix(".wav")
        .tempfile_in(folder)?;
    let mut writer = WavWriter::new(temporary.as_file_mut(), spec)?;
    let mut frames = write_buffer(&mut writer, first_buffer)?;
    frames += write_packets(format, &mut track, input, &mut writer)?;
    writer.finalize()?;
    temporary
        .persist(output)
        .map_err(|error| ExtractError::Temporary(error.error))?;
    Ok(Duration::from_secs_f64(
        frames as f64 / f64::from(spec.sample_rate),
    ))
}

fn write_packets(
    format: &mut dyn FormatReader,
    track: &mut AudioTrack,
    input: &Path,
    writer: &mut WavWriter<&mut File>,
) -> Result<u64, ExtractError> {
    let mut frames = 0_u64;
    while let Some(packet) = next_track_packet(format, track.id, input)? {
        let decoded = track
            .decoder
            .decode(&packet)
            .map_err(|error| decode_error(input, error))?;
        frames += write_buffer(writer, decoded)?;
    }
    Ok(frames)
}

fn next_track_packet(
    format: &mut dyn FormatReader,
    track_id: u32,
    input: &Path,
) -> Result<Option<Packet>, ExtractError> {
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error)) if error.kind() == ErrorKind::UnexpectedEof => {
                return Ok(None);
            }
            Err(error) => return Err(decode_error(input, error)),
        };
        if packet.track_id() == track_id {
            return Ok(Some(packet));
        }
    }
}

fn write_buffer(
    writer: &mut WavWriter<&mut File>,
    decoded: AudioBufferRef,
) -> Result<u64, ExtractError> {
    let frame_count = decoded.frames() as u64;
    let mut samples = SampleBuffer::<i16>::new(
        units::Duration::from(decoded.capacity() as u64),
        *decoded.spec(),
    );
    samples.copy_interleaved_ref(decoded);
    for sample in samples.samples() {
        writer.write_sample(*sample)?;
    }
    Ok(frame_count)
}

fn decode_error(input: &Path, error: impl std::fmt::Display) -> ExtractError {
    ExtractError::Decode {
        path: input.to_path_buf(),
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{ExtractError, extract_to_wav};
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::time::Duration;

    #[test]
    fn extracts_wav_audio_and_preserves_its_source() {
        let folder = tempfile::tempdir().unwrap();
        let input = folder.path().join("source.wav");
        let output = folder.path().join("extracted.wav");
        write_sample_wav(&input);
        let original = std::fs::read(&input).unwrap();

        assert_eq!(
            extract_to_wav(&input, &output).unwrap(),
            Duration::from_secs(1)
        );
        assert_eq!(std::fs::read(&input).unwrap(), original);
        assert_eq!(hound::WavReader::open(output).unwrap().duration(), 8_000);
    }

    #[test]
    fn reports_a_missing_video_source() {
        let folder = tempfile::tempdir().unwrap();
        let input = folder.path().join("missing.mp4");
        let output = folder.path().join("audio.wav");
        assert!(matches!(
            extract_to_wav(&input, &output),
            Err(ExtractError::MissingInput(_))
        ));
    }

    fn write_sample_wav(path: &std::path::Path) {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 8_000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(path, spec).unwrap();
        for sample in 0..8_000_i16 {
            writer.write_sample(sample % 128).unwrap();
        }
        writer.finalize().unwrap();
    }
}
