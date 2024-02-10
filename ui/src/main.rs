use backend::cv::Camera;

use eframe::egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions, ViewportBuilder};
use eframe::{egui, Frame};

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

#[derive(Default)]
struct App {
    tex_handler: Option<TextureHandle>,
    camera: Camera,
    port: Option<u8>,
}

impl App {
    fn new() -> Self {
        let mut camera = Camera::default();
        camera.connect(0).unwrap(); // TODO: handle errors

        Self {
            camera,
            ..Default::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top-row").show(ctx, |ui| {
            egui::ComboBox::from_label("Port")
                .selected_text(
                    self.port
                        .map(|v| format!("COM{v}"))
                        .unwrap_or("None".to_string()),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.port, Some(1), "COM1");
                    ui.selectable_value(&mut self.port, Some(2), "COM2");
                    ui.selectable_value(&mut self.port, Some(3), "COM3");
                });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let (size, _buf) = ([640, 480], 0)/*self.camera.get_frame().unwrap()*/;
            let texture = self.tex_handler.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "camera-frame",
                    ColorImage::new(size, Color32::LIGHT_YELLOW),
                    TextureOptions::default(),
                )
            });

            // texture.set(
            //     ImageData::Color(Arc::new(ColorImage::from_rgba_unmultiplied(size, &buf))),
            //     TextureOptions::default(),
            // );
            ui.image((texture.id(), texture.size_vec2()));
        });

        ctx.request_repaint();
    }
}
