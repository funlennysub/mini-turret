use crate::error::Error;
use backend::cv::vision::{mat_size_and_vec, to_rgba, Mat};
use backend::Turret;
use eframe::egui::{ImageData, Slider, Ui, WidgetText};
use eframe::{
    egui::{self, Color32, ColorImage, Context, TextureHandle, TextureOptions},
    Frame, Storage,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

enum Tab {
    Camera,
    CameraGray,
    CameraSettings,
    Controls,
}

impl Display for Tab {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Tab::Camera => "Camera",
            Tab::CameraGray => "Camera Gray",
            Tab::CameraSettings => "Camera Settings",
            Tab::Controls => "Controls",
        };

        write!(f, "{name}")
    }
}

#[derive(Default)]
struct MyTabViewer {
    original_frame: Option<Mat>,
    filtered_frame: Option<Mat>,

    camera_tex_handle: Option<TextureHandle>,
    gray_camera_tex_handle: Option<TextureHandle>,

    turret: Turret,

    camera_settings: CameraSettings,
    controls: (), // TODO: controls
}

impl MyTabViewer {
    fn show_camera(&mut self, ui: &mut Ui) -> crate::Result<()> {
        if self.turret.vision.source.is_none()
            || self.original_frame.is_none()
            || self.filtered_frame.is_none()
        {
            return Ok(());
        }

        self.turret
            .vision
            .get_contours(self.filtered_frame.as_ref().unwrap())?;
        self.turret
            .vision
            .find_targets(self.camera_settings.min_area)?;

        let result = self
            .turret
            .vision
            .display_info(self.original_frame.as_ref().unwrap())?;
        let (size, with_bb_frame) = mat_size_and_vec(&to_rgba(&result, 2)?)?;

        let texture = self.camera_tex_handle.as_mut().unwrap();
        texture.set(
            ImageData::Color(Arc::new(ColorImage::from_rgba_unmultiplied(
                size,
                &with_bb_frame,
            ))),
            TextureOptions::default(),
        );

        ui.image((texture.id(), texture.size_vec2()));

        Ok(())
    }

    fn show_camera_gray(&mut self, ui: &mut Ui) -> crate::Result<()> {
        if self.turret.vision.source.is_none()
            || self.original_frame.is_none()
            || self.filtered_frame.is_none()
        {
            return Ok(());
        }

        let result = &self.filtered_frame;
        let (size, frame_vec) = mat_size_and_vec(&to_rgba(result.as_ref().unwrap(), 2)?)?;

        let texture = self.gray_camera_tex_handle.as_mut().unwrap();
        texture.set(
            ImageData::Color(Arc::new(ColorImage::from_rgba_unmultiplied(
                size, &frame_vec,
            ))),
            TextureOptions::default(),
        );

        ui.image((texture.id(), texture.size_vec2()));

        Ok(())
    }
}

impl egui_dock::TabViewer for MyTabViewer {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        (*tab).to_string().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Camera => self.show_camera(ui).unwrap(),
            Tab::CameraGray => self.show_camera_gray(ui).unwrap(),
            Tab::CameraSettings => self.camera_settings.show(ui),
            _ => {}
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, Tab::Camera | Tab::Controls | Tab::CameraSettings)
    }
}

// TODO: Make it a simple egui_dock tab instead, same with default "controls"
#[derive(serde::Deserialize, serde::Serialize)]
struct CameraSettings {
    /// (H, S, V)
    upper_bound: (u8, u8, u8),
    /// (H, S, V)
    lower_bound: (u8, u8, u8),

    gray_img: bool,
    flip_frame: bool,
    min_area: f64,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            upper_bound: (255, 255, 255),
            lower_bound: (0, 0, 0),
            gray_img: false,
            flip_frame: false,
            min_area: 500.0,
        }
    }
}

impl CameraSettings {
    fn show(&mut self, ui: &mut Ui) {
        self.toggles(ui);
        self.area_size(ui);
        self.upper(ui);
        self.lower(ui);
    }

    fn toggles(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.toggle_value(&mut self.gray_img, "Gray");
            ui.toggle_value(&mut self.flip_frame, "Flip image");
        });
    }

    fn area_size(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.min_area, 0f64..=50000f64)
                .step_by(1f64)
                .text("Minimal area"),
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

pub(crate) struct App {
    context: MyTabViewer,

    error: Option<Error>,
    error_open: bool,

    tree: DockState<Tab>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            context: MyTabViewer::default(),

            error: None,
            error_open: false,

            tree: Self::create_tabs(),
        }
    }
}

impl App {
    pub(crate) fn new(cc: &eframe::CreationContext) -> Self {
        let turret = Turret::default();

        let camera_settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "cam-settings"))
            .unwrap_or_default();

        let context = MyTabViewer {
            turret,
            camera_settings,
            ..Default::default()
        };

        Self {
            context,
            ..Default::default()
        }
    }

    fn create_tabs() -> DockState<Tab> {
        let mut dock_state = DockState::new(vec![Tab::Camera]);

        let surface = dock_state.main_surface_mut();

        let [_old, _new] = surface.split_right(NodeIndex::root(), 0.5, vec![Tab::CameraGray]);

        let [_old, _new] = surface.split_below(
            NodeIndex::root(),
            0.69,
            vec![Tab::Controls, Tab::CameraSettings],
        );

        dock_state
    }

    fn reset_tabs(&mut self, ui: &mut Ui) {
        if ui.button("Reset tabs").clicked() {
            self.tree = Self::create_tabs();
        }
    }

    fn connect_cam(&mut self, ui: &mut Ui) -> crate::Result<()> {
        match &self.context.turret.vision.source {
            None => {
                if ui.button("Connect camera").clicked() {
                    self.context.turret.vision.connect(0)?;
                }
            }
            Some(_) => {
                if ui.button("Disconnect camera").clicked() {
                    self.context.turret.vision.disconnect().unwrap();
                }
            }
        }

        Ok(())
    }

    fn top_bar(&mut self, ui: &mut Ui) -> crate::Result<()> {
        ui.horizontal(|ui| -> crate::Result<()> {
            self.connect_cam(ui)?;
            self.reset_tabs(ui);

            Ok(())
        })
        .inner?;

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

        if self.context.turret.vision.source.is_some() {
            let frame = self
                .context
                .turret
                .vision
                .get_frame(self.context.camera_settings.flip_frame)?;

            let (size, _) = mat_size_and_vec(&to_rgba(&frame, 2)?)?;

            self.context.camera_tex_handle = Some(ctx.load_texture(
                "camera",
                ColorImage::new(size, Color32::LIGHT_YELLOW),
                TextureOptions::default(),
            ));
            self.context.gray_camera_tex_handle = Some(ctx.load_texture(
                "gray-camera",
                ColorImage::new(size, Color32::LIGHT_YELLOW),
                TextureOptions::default(),
            ));

            self.context.filtered_frame = Some(self.context.turret.vision.filter_color(
                &frame,
                self.context.camera_settings.lower_bound,
                self.context.camera_settings.upper_bound,
            )?);

            self.context.original_frame = Some(frame);
        }

        egui::TopBottomPanel::top("top-row")
            .show(ctx, |ui| self.top_bar(ui))
            .inner?;

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                DockArea::new(&mut self.tree)
                    .style(Style::from_egui(ctx.style().as_ref()))
                    .show_inside(ui, &mut self.context);
            });

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
        eframe::set_value(storage, "cam-settings", &self.context.camera_settings);
    }
}
