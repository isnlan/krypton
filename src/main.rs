mod models;
mod core;
mod ui;
mod app;

use app::KryptonApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("File Encryption/Decryption Tool"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Krypton - File Encryption Tool",
        options,
        Box::new(|_cc| {
            Ok(Box::new(KryptonApp::new()))
        }),
    )
}
