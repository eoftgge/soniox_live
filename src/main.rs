#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{IconData, ViewportBuilder};
use eframe::icon_data::from_png_bytes;
use soniox_live::errors::SonioxLiveErrors;
use soniox_live::gui::fonts::setup_custom_fonts;
use soniox_live::settings::SettingsApp;
use soniox_live::setup_tracing;
use soniox_live::gui::app::SubtitlesApp;

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

fn run() -> Result<(), SonioxLiveErrors> {
    let settings = SettingsApp::new("soniox.toml")?;
    let level = settings.level();
    let guard = setup_tracing(level, settings.log_to_file());
    let app = SubtitlesApp::new(settings, guard);

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id("sublive")
            .with_icon(from_png_bytes(ICON_BYTES).unwrap_or_else(|_| {
                tracing::warn!("Bytes of icon is incorrect...");
                IconData::default()
            }))
            .with_inner_size([400., 600.])
            .with_resizable(false)
            .with_decorations(true)
            .with_always_on_top()
            .with_transparent(true)
            .with_maximize_button(false),
        ..Default::default()
    };

    tracing::info!("Starting application");
    let res = eframe::run_native(
        "Soniox Live",
        native_options,
        Box::new(move |cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );
    if let Err(e) = res {
        tracing::error!("err: {}", e);
    }

    Ok(())
}

fn main() {
    #[cfg(target_os = "macos")]
    embed_plist::embed_info_plist!("Info.plist");

    let rt = tokio::runtime::Runtime::new().expect("Should be able to get rt main thread");
    let _e = rt.enter();

    if let Err(err) = run() {
        eprintln!("Soniox Live {:?}", err);
        std::process::exit(1);
    }
}
