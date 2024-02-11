use crate::error::Error;
use backend::cv::vision::{mat_size_and_vec, to_rgba};
use backend::{list_devices, Turret};
use eframe::egui::{ImageData, Slider, Ui};
use eframe::{
    egui::{self, Color32, ColorImage, Context, TextureHandle, TextureOptions},
    Frame, Storage,
};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
struct CameraSettings {
    /// (H, S, V)
    upper_bound: (u8, u8, u8),
    /// (H, S, V)
    lower_bound: (u8, u8, u8),

    gray_img: bool,
    flip_frame: bool,
    min_bb_size: f64,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            upper_bound: (255, 255, 255),
            lower_bound: (0, 0, 0),
            gray_img: false,
            flip_frame: false,
            min_bb_size: 500.0,
        }
    }
}

impl CameraSettings {
    fn toggles(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.toggle_value(&mut self.gray_img, "Gray");
            ui.toggle_value(&mut self.flip_frame, "Flip image");
        });
    }

    fn bb_size(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.min_bb_size, 0f64..=50000f64)
                .step_by(1f64)
                .text("BB Size"),
        );
    }
    fn upper(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.monospace("Upper bound:");
            ui.add(Slider::new(&mut self.upper_bound.0, 0..=255).text("Hue"));
            ui.add(Slider::new(&mut self.upper_bound.1, 0..=255).text("Saturation"));
            ui.add(Slider::new(&mut self.upper_bound.2, 0..=255).text("Value"));
        });
    }

    fn lower(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.monospace("Lower bound:");
            ui.add(Slider::new(&mut self.lower_bound.0, 0..=255).text("Hue"));
            ui.add(Slider::new(&mut self.lower_bound.1, 0..=255).text("Saturation"));
            ui.add(Slider::new(&mut self.lower_bound.2, 0..=255).text("Value"));
        });
    }
}

#[derive(Default)]
pub(crate) struct App {
    tex_handler: Option<TextureHandle>,
    turret: Turret,
    port: Option<PathBuf>,

    cam_settings_open: bool,
    cam_settings: CameraSettings,

    error: Option<Error>,
    error_open: bool,
}

impl App {
    pub(crate) fn new(cc: &eframe::CreationContext) -> Self {
        let mut turret = Turret::default();
        turret.vision.connect(0).unwrap();

        let calibrator = cc
            .storage
            .and_then(|s| eframe::get_value(s, "cam-settings"))
            .unwrap_or_default();

        Self {
            turret,
            cam_settings: calibrator,
            ..Default::default()
        }
    }

    fn port_picker(&mut self, ui: &mut Ui) {
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
    }

    fn calibrate_btn(&mut self, ui: &mut Ui) {
        if ui.button("Calibrate color").clicked() {
            self.cam_settings_open = !self.cam_settings_open;
        }
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.port_picker(ui);
            self.calibrate_btn(ui)
        });
    }

    fn camera_settings(&mut self, ui: &mut Ui) {
        self.cam_settings.toggles(ui);
        self.cam_settings.bb_size(ui);
        self.cam_settings.upper(ui);
        self.cam_settings.lower(ui);
    }

    fn controls(&mut self, _ui: &mut Ui) {}

    fn central_panel(&mut self, ui: &mut Ui) -> crate::Result<()> {
        let frame = self.turret.vision.get_frame(self.cam_settings.flip_frame)?;
        let (size, _) = mat_size_and_vec(&to_rgba(&frame, 2)?)?;
        let texture = self.tex_handler.get_or_insert_with(|| {
            ui.ctx().load_texture(
                "camera-frame",
                ColorImage::new(size, Color32::LIGHT_YELLOW),
                TextureOptions::default(),
            )
        });

        let filtered_frame = self.turret.vision.filter_color(
            &frame,
            self.cam_settings.lower_bound,
            self.cam_settings.upper_bound,
        )?;
        self.turret.vision.get_contours(&filtered_frame)?;

        let result = if self.cam_settings.gray_img {
            filtered_frame
        } else {
            self.turret
                .vision
                .draw_bb(&frame, self.cam_settings.min_bb_size)?
        };

        let (size, with_bb_frame) = mat_size_and_vec(&to_rgba(&result, 2)?)?;
        texture.set(
            ImageData::Color(Arc::new(ColorImage::from_rgba_unmultiplied(
                size,
                &with_bb_frame,
            ))),
            TextureOptions::default(),
        );

        ui.image((texture.id(), texture.size_vec2()));

        match self.cam_settings_open {
            true => self.camera_settings(ui),
            false => self.controls(ui),
        }

        Ok(())
    }

    fn show_err(&mut self, ctx: &Context) {
        egui::Window::new("Error").show(ctx, |ui| {
            ui.label("An error was encountered:");
            ui.monospace(self.error.as_ref().unwrap().to_string());
            ui.horizontal(|ui| {
                if ui.button("Ok").clicked() {
                    self.error_open = false
                }
            });
        });
    }

    fn app(&mut self, ctx: &Context, _frame: &mut Frame) -> crate::Result<()> {
        if self.error_open {
            self.show_err(ctx);
        }

        egui::TopBottomPanel::top("top-row").show(ctx, |ui| self.top_bar(ui));

        egui::CentralPanel::default()
            .show(ctx, |ui| -> crate::Result<()> { self.central_panel(ui) })
            .inner?;

        Ok(())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        match self.app(ctx, _frame) {
            Err(err) if !self.error_open => {
                self.error_open = true;
                println!("openeds");
                self.error = Some(err);
            }
            _ => {}
        }

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "cam-settings", &self.cam_settings);
    }
}
