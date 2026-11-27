//! Saving small text files that outlive the app: the settings, saved games and best times.
//!
//! On the web they go in the browser's local storage (under `bevy-solitaire-<name>`); on desktop,
//! in a `bevy-solitaire` folder in the user's config folder (as `<name>.txt`).
pub use platform::{load, remove, save};

#[cfg(target_arch = "wasm32")]
mod platform {
    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    fn key(name: &str) -> String {
        format!("bevy-solitaire-{name}")
    }

    /// The text saved as `name`, if any.
    pub fn load(name: &str) -> Option<String> {
        local_storage()?.get_item(&key(name)).ok()?
    }

    /// Saves `text` as `name`, replacing what was there.
    pub fn save(name: &str, text: &str) {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(&key(name), text);
        }
    }

    /// Forgets the text saved as `name`.
    pub fn remove(name: &str) {
        if let Some(storage) = local_storage() {
            let _ = storage.remove_item(&key(name));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use std::{env, fs, path::PathBuf};

    fn path(name: &str) -> Option<PathBuf> {
        let config = env::var_os("APPDATA")
            .or_else(|| env::var_os("XDG_CONFIG_HOME"))
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(config.join("bevy-solitaire").join(format!("{name}.txt")))
    }

    /// The text saved as `name`, if any.
    pub fn load(name: &str) -> Option<String> {
        fs::read_to_string(path(name)?).ok()
    }

    /// Saves `text` as `name`, replacing what was there.
    pub fn save(name: &str, text: &str) {
        let Some(path) = path(name) else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Err(error) = fs::write(&path, text) {
            bevy::log::warn!("Couldn't save {}: {error}", path.display());
        }
    }

    /// Forgets the text saved as `name`.
    pub fn remove(name: &str) {
        if let Some(path) = path(name) {
            let _ = fs::remove_file(path);
        }
    }
}
