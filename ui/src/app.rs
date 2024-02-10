use backend::{list_devices, Turret};
use eframe::{
    egui::{self, Color32, ColorImage, Context, TextureHandle, TextureOptions},
    Frame,
};
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct App {
    tex_handler: Option<TextureHandle>,
    turret: Turret,
    port: Option<PathBuf>,
}

impl App {
    pub(crate) fn new() -> Self {
        let mut turret = Turret::default();
        turret.vision.connect(0).unwrap();

        Self {
            turret,
            ..Default::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top-row").show(ctx, |ui| {
            let changed = egui::ComboBox::from_label("Port")
                .selected_text(
                    self.port
                        .as_ref()
                        .map(|v| v.to_string_lossy().to_string())
                        .unwrap_or("None".to_string()),
                )
                .show_ui(ui, |ui| {
                    for port in list_devices().unwrap() {
                        if ui
                            .selectable_value(
                                &mut self.port,
                                Some(port.to_owned()),
                                format!("{}", port.display()),
                            )
                            .clicked()
                        {
                            return true;
                        }
                    }
                    false
                });
            if let Some(changed) = changed.inner {
                if changed {
                    println!("CHanged port to {:?}", &self.port)
                }
            }
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
