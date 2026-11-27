//! Buttons that open a web page.
use bevy::prelude::*;

/// A button that opens this URL when pressed.
#[derive(Component)]
pub struct Link(pub &'static str);

/// Opens a link button's page when the pointer is released over it.
///
/// Browsers only let pages open new tabs in response to the user, and on touch screens it is the
/// release, not the press, that counts. This runs shortly after the release, which browsers
/// allow.
pub fn on_link_press(release: On<PointerRelease>, links: Query<&Link>) {
    if let Ok(link) = links.get(release.entity) {
        info!("Opening {}", link.0);
        open(link.0);
    }
}

/// Opens `url` in a new browser tab, or in this one if new tabs are blocked.
#[cfg(target_arch = "wasm32")]
fn open(url: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let opened = window
        .open_with_url_and_target(url, "_blank")
        .ok()
        .flatten();
    if opened.is_none() {
        let _ = window.location().set_href(url);
    }
}

/// Opens `url` in the default web browser.
#[cfg(not(target_arch = "wasm32"))]
fn open(url: &str) {
    use std::process::Command;
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", url]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else {
        Command::new("xdg-open").arg(url).spawn()
    };
    if let Err(error) = result {
        warn!("Couldn't open {url}: {error}");
    }
}
