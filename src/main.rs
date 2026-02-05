mod about;
mod app;
mod config;
mod discovery;
mod esi;
mod process;
mod settings;
mod theme;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 450.0])
            .with_min_inner_size([700.0, 350.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Pack Preferences",
        options,
        Box::new(|cc| Ok(Box::new(app::PackPreferencesApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;

    Ok(())
}
