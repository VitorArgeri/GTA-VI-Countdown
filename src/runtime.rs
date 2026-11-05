use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};

use chrono::Utc;
use slint::{Color, ComponentHandle, Timer, TimerMode, Weak};
use thiserror::Error;

use crate::{
    audio::{AudioError, AudioPlayback},
    countdown::{CountdownState, countdown_at},
    settings::{Preferences, load_preferences, preferences_path, save_preferences},
    wallpapers::{Wallpaper, WallpaperError, load_wallpapers},
};

slint::include_modules!();

const ACCENTS: [(u8, u8, u8); 5] = [
    (232, 137, 178),
    (119, 207, 200),
    (227, 173, 123),
    (174, 157, 204),
    (232, 216, 187),
];

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Não foi possível abrir a janela: {0}")]
    Window(#[from] slint::PlatformError),
    #[error(transparent)]
    Wallpapers(#[from] WallpaperError),
}

pub fn run() -> Result<(), RuntimeError> {
    let assets = asset_folder();
    let wallpapers = load_wallpapers(&assets.join("wallpapers"))?;
    let settings_path = preferences_path();
    let preferences = Rc::new(RefCell::new(load_preferences(&settings_path)));
    let audio_path = assets.join("audio").join("love-is-a-long-road.wav");
    let (audio, audio_message) = open_audio(audio_path, &preferences.borrow());
    let window = CountdownWindow::new()?;
    configure_window(&window, &wallpapers, &preferences.borrow(), &audio_message);
    connect_wallpaper_control(&window, &wallpapers, &preferences, &settings_path);
    connect_appearance_controls(&window, &preferences, &settings_path);
    connect_audio_controls(&window, &audio, &preferences, &settings_path);
    connect_fullscreen_controls(&window);
    let _timer = start_timer(&window, &audio);
    window.run()?;
    Ok(())
}

fn asset_folder() -> PathBuf {
    let beside_exe = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|folder| folder.join("assets")));
    beside_exe
        .filter(|folder| folder.is_dir())
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
}

fn open_audio(
    path: PathBuf,
    settings: &Preferences,
) -> (Rc<RefCell<Option<AudioPlayback>>>, String) {
    match AudioPlayback::open(path) {
        Ok(mut audio) => {
            audio.change_volume(settings.volume - 0.35);
            if settings.muted {
                audio.toggle_mute();
            }
            (Rc::new(RefCell::new(Some(audio))), String::new())
        }
        Err(error) => (Rc::new(RefCell::new(None)), audio_message(error)),
    }
}

fn audio_message(error: AudioError) -> String {
    format!("ÁUDIO INDISPONÍVEL · {error}")
}

fn configure_window(
    window: &CountdownWindow,
    wallpapers: &[Wallpaper],
    settings: &Preferences,
    audio_message: &str,
) {
    window.set_wallpaper(wallpapers[settings.wallpaper].image.clone());
    window.set_wallpaper_one(wallpapers[0].image.clone());
    window.set_wallpaper_two(wallpapers[1].image.clone());
    window.set_wallpaper_three(wallpapers[2].image.clone());
    window.set_wallpaper_index(settings.wallpaper as i32);
    window.set_accent_index(settings.accent as i32);
    window.set_timer_accent(accent_color(settings.accent));
    window.set_timer_scale(settings.scale);
    window.set_position_x(settings.position_x);
    window.set_position_y(settings.position_y);
    window.set_active_anchor(settings.anchor);
    window.set_muted(settings.muted);
    window.set_volume_label(format!("{:.0}%", settings.volume * 100.0).into());
    window.set_audio_message(audio_message.into());
}

fn accent_color(index: usize) -> Color {
    let (red, green, blue) = ACCENTS[index.min(ACCENTS.len() - 1)];
    Color::from_rgb_u8(red, green, blue)
}

fn connect_wallpaper_control(
    window: &CountdownWindow,
    wallpapers: &[Wallpaper],
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
) {
    let images = wallpapers
        .iter()
        .map(|wallpaper| wallpaper.image.clone())
        .collect::<Vec<_>>();
    let weak = window.as_weak();
    let preferences = Rc::clone(preferences);
    let path = path.to_path_buf();
    window.on_select_wallpaper(move |index| {
        let Some(image) = images.get(index as usize) else {
            return;
        };
        if let Some(window) = weak.upgrade() {
            window.set_wallpaper(image.clone());
            window.set_wallpaper_index(index);
            window.set_wallpaper_sidebar_open(false);
        }
        preferences.borrow_mut().wallpaper = index as usize;
        if let Some(window) = weak.upgrade() {
            persist_preferences(&window, &preferences, &path);
        }
    });
    connect_sidebar_toggles(window);
}

fn connect_sidebar_toggles(window: &CountdownWindow) {
    let weak = window.as_weak();
    window.on_toggle_wallpaper_sidebar({
        let weak = weak.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let open = !window.get_wallpaper_sidebar_open();
                window.set_wallpaper_sidebar_open(open);
                window.set_settings_sidebar_open(false);
            }
        }
    });
    window.on_toggle_settings_sidebar({
        let weak = weak.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let open = !window.get_settings_sidebar_open();
                window.set_settings_sidebar_open(open);
                window.set_wallpaper_sidebar_open(false);
            }
        }
    });
    window.on_close_sidebars(move || {
        if let Some(window) = weak.upgrade() {
            window.set_wallpaper_sidebar_open(false);
            window.set_settings_sidebar_open(false);
        }
    });
}

fn connect_appearance_controls(
    window: &CountdownWindow,
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
) {
    connect_anchor_control(window, preferences, path);
    connect_accent_control(window, preferences, path);
    connect_scale_controls(window, preferences, path);
    let weak = window.as_weak();
    let preferences = Rc::clone(preferences);
    let path = path.to_path_buf();
    window.on_save_dragged_position(move || {
        let Some(window) = weak.upgrade() else { return };
        let mut settings = preferences.borrow_mut();
        settings.position_x = window.get_position_x();
        settings.position_y = window.get_position_y();
        settings.anchor = -1;
        drop(settings);
        persist_preferences(&window, &preferences, &path);
    });
}

fn connect_anchor_control(
    window: &CountdownWindow,
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
) {
    let weak = window.as_weak();
    let preferences = Rc::clone(preferences);
    let path = path.to_path_buf();
    window.on_set_anchor(move |index| {
        let Some((x, y)) = anchor_position(index) else {
            return;
        };
        let Some(window) = weak.upgrade() else { return };
        window.set_position_x(x);
        window.set_position_y(y);
        window.set_active_anchor(index);
        let mut settings = preferences.borrow_mut();
        settings.position_x = x;
        settings.position_y = y;
        settings.anchor = index;
        drop(settings);
        persist_preferences(&window, &preferences, &path);
    });
}

fn anchor_position(index: i32) -> Option<(f32, f32)> {
    const AXES: [f32; 3] = [0.0, 0.5, 1.0];
    if !(0..=8).contains(&index) {
        return None;
    }
    Some((AXES[(index % 3) as usize], AXES[(index / 3) as usize]))
}

fn connect_accent_control(
    window: &CountdownWindow,
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
) {
    let weak = window.as_weak();
    let preferences = Rc::clone(preferences);
    let path = path.to_path_buf();
    window.on_choose_accent(move |index| {
        if !(0..ACCENTS.len() as i32).contains(&index) {
            return;
        }
        let Some(window) = weak.upgrade() else { return };
        window.set_accent_index(index);
        window.set_timer_accent(accent_color(index as usize));
        preferences.borrow_mut().accent = index as usize;
        persist_preferences(&window, &preferences, &path);
    });
}

fn connect_scale_controls(
    window: &CountdownWindow,
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
) {
    for (increase, delta) in [(false, -0.05), (true, 0.05)] {
        let weak = window.as_weak();
        let preferences = Rc::clone(preferences);
        let path = path.to_path_buf();
        if increase {
            window.on_scale_up(move || change_scale(&weak, &preferences, &path, delta));
        } else {
            window.on_scale_down(move || change_scale(&weak, &preferences, &path, delta));
        }
    }
}

fn change_scale(
    window: &Weak<CountdownWindow>,
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
    delta: f32,
) {
    let Some(window) = window.upgrade() else {
        return;
    };
    let scale = (window.get_timer_scale() + delta).clamp(0.7, 1.1);
    window.set_timer_scale(scale);
    preferences.borrow_mut().scale = scale;
    persist_preferences(&window, preferences, path);
}

fn connect_audio_controls(
    window: &CountdownWindow,
    audio: &Rc<RefCell<Option<AudioPlayback>>>,
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
) {
    let weak = window.as_weak();
    let audio_for_mute = Rc::clone(audio);
    let preferences_for_mute = Rc::clone(preferences);
    let path_for_mute = path.to_path_buf();
    window.on_toggle_mute(move || {
        let Some(muted) = audio_for_mute
            .borrow_mut()
            .as_mut()
            .map(AudioPlayback::toggle_mute)
        else {
            return;
        };
        let Some(window) = weak.upgrade() else { return };
        window.set_muted(muted);
        preferences_for_mute.borrow_mut().muted = muted;
        persist_preferences(&window, &preferences_for_mute, &path_for_mute);
    });
    connect_volume_control(
        window,
        Rc::clone(audio),
        Rc::clone(preferences),
        path.to_path_buf(),
    );
}

fn connect_volume_control(
    window: &CountdownWindow,
    audio: Rc<RefCell<Option<AudioPlayback>>>,
    preferences: Rc<RefCell<Preferences>>,
    path: PathBuf,
) {
    for (increase, delta) in [(false, -0.05), (true, 0.05)] {
        let weak = window.as_weak();
        let audio = Rc::clone(&audio);
        let preferences = Rc::clone(&preferences);
        let path = path.clone();
        let callback = move || {
            let Some(volume) = audio
                .borrow_mut()
                .as_mut()
                .map(|player| player.change_volume(delta))
            else {
                return;
            };
            let Some(window) = weak.upgrade() else { return };
            window.set_volume_label(format!("{:.0}%", volume * 100.0).into());
            preferences.borrow_mut().volume = volume;
            persist_preferences(&window, &preferences, &path);
        };
        if increase {
            window.on_volume_up(callback);
        } else {
            window.on_volume_down(callback);
        }
    }
}

fn persist_preferences(
    window: &CountdownWindow,
    preferences: &Rc<RefCell<Preferences>>,
    path: &Path,
) {
    match save_preferences(path, &preferences.borrow()) {
        Ok(()) => window.set_settings_message("".into()),
        Err(error) => {
            window.set_settings_message(format!("Não foi possível salvar: {error}").into())
        }
    }
}

fn connect_fullscreen_controls(window: &CountdownWindow) {
    let weak = window.as_weak();
    window.on_request_fullscreen({
        let weak = weak.clone();
        move || set_fullscreen(&weak, true)
    });
    window.on_request_windowed(move || set_fullscreen(&weak, false));
}

fn set_fullscreen(window: &Weak<CountdownWindow>, fullscreen: bool) {
    if let Some(window) = window.upgrade() {
        window.window().set_fullscreen(fullscreen);
    }
}

fn start_timer(window: &CountdownWindow, audio: &Rc<RefCell<Option<AudioPlayback>>>) -> Timer {
    let weak = window.as_weak();
    let audio = Rc::clone(audio);
    let started_at = Instant::now();
    let mut last_second = i64::MIN;
    let mut launched = false;
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        let Some(window) = weak.upgrade() else { return };
        update_countdown(
            &window,
            &mut last_second,
            &mut launched,
            &audio,
            started_at.elapsed(),
        );
    });
    timer
}

fn update_countdown(
    window: &CountdownWindow,
    last_second: &mut i64,
    launched: &mut bool,
    audio: &Rc<RefCell<Option<AudioPlayback>>>,
    elapsed: Duration,
) {
    let now = Utc::now();
    if now.timestamp() != *last_second {
        update_clock(window, now, last_second);
    }
    if matches!(countdown_at(now), CountdownState::Launched) {
        begin_celebration(window, launched, audio);
        let pulse = (elapsed.as_secs_f64() * 2.5).sin().mul_add(0.5, 0.5) as f32;
        window.set_pulse(pulse);
    }
}

fn update_clock(window: &CountdownWindow, now: chrono::DateTime<Utc>, last_second: &mut i64) {
    *last_second = now.timestamp();
    match countdown_at(now) {
        CountdownState::Counting(time) => {
            window.set_days(format!("{:02}", time.days).into());
            window.set_hours(format!("{:02}", time.hours).into());
            window.set_minutes(format!("{:02}", time.minutes).into());
            window.set_seconds(format!("{:02}", time.seconds).into());
        }
        CountdownState::Launched => {
            window.set_days("00".into());
            window.set_hours("00".into());
            window.set_minutes("00".into());
            window.set_seconds("00".into());
        }
    }
}

fn begin_celebration(
    window: &CountdownWindow,
    launched: &mut bool,
    audio: &Rc<RefCell<Option<AudioPlayback>>>,
) {
    if *launched {
        return;
    }
    *launched = true;
    window.set_launched(true);
    if let Some(audio) = audio.borrow_mut().as_mut()
        && let Err(error) = audio.start_loop()
    {
        window.set_audio_message(audio_message(error).into());
    }
}

#[cfg(test)]
mod tests {
    use super::anchor_position;

    #[test]
    fn nine_alignment_choices_cover_all_screen_anchors() {
        assert_eq!(anchor_position(0), Some((0.0, 0.0)));
        assert_eq!(anchor_position(4), Some((0.5, 0.5)));
        assert_eq!(anchor_position(8), Some((1.0, 1.0)));
        assert_eq!(anchor_position(9), None);
        assert_eq!(anchor_position(-1), None);
    }
}
