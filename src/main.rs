#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod cache;
mod file_info;
pub mod lang;
mod scanner;
mod ui;
mod utils;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title("Folder Cleaner")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Folder Cleaner",
        options,
        Box::new(|cc| Ok(Box::new(app::FolderCleanerApp::new(cc)))),
    )
}

fn load_icon() -> eframe::egui::IconData {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon from memory")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    eframe::egui::IconData {
        rgba,
        width,
        height,
    }
}
