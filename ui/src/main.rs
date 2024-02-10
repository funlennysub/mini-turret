use crate::app::App;

use eframe::egui::ViewportBuilder;

mod app;
mod error;

const MIN_SIZE: [f32; 2] = [650.0, 650.0];

fn main() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_min_inner_size(MIN_SIZE),
        ..Default::default()
    };

    eframe::run_native("Mini-Turret", options, Box::new(|_cc| Box::new(App::new())))
        .expect("TODO: panic message");
}
