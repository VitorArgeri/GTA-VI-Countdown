use std::path::{Path, PathBuf};

use slint::Image;
use thiserror::Error;

#[derive(Clone)]
pub struct Wallpaper {
    pub label: &'static str,
    pub path: PathBuf,
    pub image: Image,
}

#[derive(Debug, Error)]
pub enum WallpaperError {
    #[error("Wallpaper ausente em '{0}'; esperado JPG ou PNG em assets/wallpapers.")]
    Missing(PathBuf),
    #[error("Não foi possível abrir o wallpaper '{path}': {reason}")]
    Invalid { path: PathBuf, reason: String },
}

const WALLPAPER_FILES: [(&str, &str); 3] = [
    ("JASON & LUCIA · ASSALTO", "01_rockstar_robbery.jpg"),
    ("JASON & LUCIA · VICE CITY", "02_rockstar_jason_lucia.jpg"),
    ("VICE CITY · PÔR DO SOL", "03_alphacoders_1343606.png"),
];

pub fn load_wallpapers(folder: &Path) -> Result<Vec<Wallpaper>, WallpaperError> {
    WALLPAPER_FILES
        .iter()
        .map(|(label, name)| load_wallpaper(folder, label, name))
        .collect()
}

fn load_wallpaper(
    folder: &Path,
    label: &'static str,
    name: &str,
) -> Result<Wallpaper, WallpaperError> {
    let path = folder.join(name);
    if !path.is_file() {
        return Err(WallpaperError::Missing(path));
    }
    let image = Image::load_from_path(&path).map_err(|error| WallpaperError::Invalid {
        path: path.clone(),
        reason: error.to_string(),
    })?;
    Ok(Wallpaper { label, path, image })
}

#[cfg(test)]
mod tests {
    use super::load_wallpapers;
    use std::path::Path;

    #[test]
    fn loads_three_local_wallpapers() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/wallpapers");
        let wallpapers = load_wallpapers(&folder).expect("the three bundled wallpapers load");
        assert_eq!(wallpapers.len(), 3);
    }
}
