use std::{env, path::PathBuf, process::ExitCode};

use gta_vi_countdown::extract::extract_to_wav;

fn main() -> ExitCode {
    match extract_arguments() {
        Ok((input, output)) => print_result(extract_to_wav(&input, &output)),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn extract_arguments() -> Result<(PathBuf, PathBuf), String> {
    let mut arguments = env::args_os().skip(1);
    let input = arguments.next().map(PathBuf::from);
    let output = arguments.next().map(PathBuf::from);
    if arguments.next().is_some() || input.is_none() || output.is_none() {
        return Err("Uso: cargo run --bin extract-audio -- <video.mp4> <saida.wav>".to_owned());
    }
    Ok((
        input.expect("checked input"),
        output.expect("checked output"),
    ))
}

fn print_result(
    result: Result<std::time::Duration, gta_vi_countdown::extract::ExtractError>,
) -> ExitCode {
    match result {
        Ok(duration) => {
            println!(
                "Áudio extraído e validado: {:.0} segundos.",
                duration.as_secs_f64()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Não foi possível extrair o áudio: {error}");
            ExitCode::FAILURE
        }
    }
}
