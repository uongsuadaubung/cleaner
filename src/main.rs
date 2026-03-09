#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod file_info;
mod scanner;
mod ui;
mod utils;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title("Folder Cleaner"),
        ..Default::default()
    };

    eframe::run_native(
        "Folder Cleaner",
        options,
        Box::new(|cc| Ok(Box::new(app::FolderCleanerApp::new(cc)))),
    )
}
