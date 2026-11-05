#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = gta_vi_countdown::runtime::run() {
        eprintln!("Não foi possível iniciar o Timer GTA VI: {error}");
        std::process::exit(1);
    }
}
