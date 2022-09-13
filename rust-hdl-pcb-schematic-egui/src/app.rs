use eframe::{egui, epi};
use eframe::egui::{CentralPanel, Color32, Context, CursorIcon, Frame, PointerButton, Pos2, Rect, Sense, Shape, Stroke, vec2, Vec2};
use eframe::egui::emath::{RectTransform, Rot2};
use eframe::emath::{pos2, remap};
use eframe::epaint::Rounding;
use eframe::epi::Storage;
use rust_hdl_pcb::adc::make_ads868x;
use rust_hdl_pcb_core::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DocBounds {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl DocBounds {
    pub fn min(&self) -> [f32; 2] {
        self.min
    }
    pub fn max(&self) -> [f32; 2] {
        self.max
    }
    pub fn is_valid(&self) -> bool {
        self.width() > 0.0 && self.height() > 0.0
    }
    pub fn width(&self) -> f32 {
        self.max[0] - self.min[0]
    }
    pub fn height(&self) -> f32 {
        self.max[1] - self.min[1]
    }
    pub fn center(&self) -> Vec2 {
        Vec2 {
            x: (self.min[0] + self.max[0]) / 2.0,
            y: (self.min[1] + self.max[1]) / 2.0,
        }
    }
    pub fn translate_x(&mut self, delta: f32) {
        self.min[0] += delta;
        self.max[0] += delta;
    }
    pub fn translate_y(&mut self, delta: f32) {
        self.min[1] += delta;
        self.max[1] += delta;
    }
    pub fn translate(&mut self, delta: Vec2) {
        self.translate_x(delta.x);
        self.translate_y(delta.y);
    }
}

pub struct ScreenTransform {
    frame: Rect,
    bounds: DocBounds,
}

impl ScreenTransform {
    pub fn new(frame: Rect, bounds: DocBounds) -> Self {
        Self {
            frame,
            bounds
        }
    }
    pub fn frame(&self) -> &Rect {
        &self.frame
    }
    pub fn bounds(&self) -> &DocBounds {
        &self.bounds
    }
    pub fn bounds_mut(&mut self) -> &mut DocBounds {
        &mut self.bounds
    }
    pub fn position_from_value(&self, value: &Pos2) -> Pos2 {
        let x = remap(value.x,
                      self.bounds.min[0]..=self.bounds.max[0],
                      self.frame.left()..=self.frame.right()
        );
        let y = remap(value.y,
                      self.bounds.min[1]..=self.bounds.max[1],
                      self.frame.bottom()..=self.frame.top()
        );
        pos2(x, y)
    }
    pub fn value_from_position(&self, pos: &Pos2) -> Pos2 {
        let x = remap(pos.x,
                      self.frame.left()..=self.frame.right(),
                      self.bounds.min[0]..=self.bounds.max[0]
        );
        let y = remap(pos.y,
                      self.frame.bottom()..=self.frame.top(),
                      self.bounds.min[1]..=self.bounds.max[1],
        );
        pos2(x, y)
    }
    pub fn translate_bounds(&mut self, mut delta_pos: Vec2) {
        delta_pos.x *= self.dvalue_dpos()[0];
        delta_pos.y *= self.dvalue_dpos()[1];
        self.bounds.translate(delta_pos);
    }
    pub fn dpos_dvalue_x(&self) -> f32 {
        self.frame.width() / self.bounds.width()
    }
    pub fn dpos_dvalue_y(&self) -> f32 {
        -self.frame.height() / self.bounds.height()
    }
    pub fn dpos_dvalue(&self) -> [f32; 2] {
        [self.dpos_dvalue_x(), self.dpos_dvalue_y()]
    }
    pub fn dvalue_dpos(&self) -> [f32; 2] {
        [1.0/self.dpos_dvalue_x(), 1.0/self.dpos_dvalue_y()]
    }
    pub fn rect_from_values(&self, value1: &Pos2, value2: &Pos2) -> Rect {
        let pos1 = self.position_from_value(value1);
        let pos2 = self.position_from_value(value2);
        let mut rect = Rect::NOTHING;
        rect.extend_with(pos1);
        rect.extend_with(pos2);
        rect
    }
}



/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
//    label: String,

    rotation: f32,
    translation: Vec2,
    zoom: f32,
    circuit: Circuit,
    layout: SchematicLayout,
}


impl Default for TemplateApp {
    fn default() -> Self {
        let (mut circuit, mut layout) = rust_hdl_pcb::schematic_manual_layout::test_ldo_circuit();
        circuit
            .nodes
            .push(make_ads868x("ADS8681IPW").instance("adc"));
        layout.set_part("adc", orient().center(4000, 4000));
        Self {
            // Example stuff:
            rotation: 0.0,
            translation: Default::default(),
            zoom: 1.0,
            circuit,
            layout
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "eframe starter"
    }

    /// Called once before the first frame.
    fn setup(&mut self, _ctx: &Context, _frame: &epi::Frame, _storage: Option<&dyn Storage>) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }


    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &Context, _frame: &epi::Frame) {
        /*
        let Self { label, value } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }

         */
        /*
        CentralPanel::default().show(ctx, |ui| {
            Frame::dark_canvas(ui.style()).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

                    let painter_proportions = response.rect.square_proportions();
                    let to_screen = RectTransform::from_to(
                        Rect::from_min_size(Pos2::ZERO - painter_proportions, 2.0 * painter_proportions),
                        response.rect
                    );

                    // check for touch input (or the lack thereof) and update zoom and scale factors, plus
                    // color and width:
                    let mut stroke_width = 1.;
                    let color = Color32::GRAY;
                    if let Some(multi_touch) = ui.input().multi_touch() {
                        // This adjusts the current zoom factor and rotation angle according to the dynamic
                        // change (for the current frame) of the touch gesture:
                        self.zoom *= multi_touch.zoom_delta;
                        self.rotation += multi_touch.rotation_delta;
                        // the translation we get from `multi_touch` needs to be scaled down to the
                        // normalized coordinates we use as the basis for painting:
                        self.translation += to_screen.inverse().scale() * multi_touch.translation_delta;
                        // touch pressure will make the arrow thicker (not all touch devices support this):
                        stroke_width += 10. * multi_touch.force;
                    }
                    let zoom_and_rotate = self.zoom * Rot2::from_angle(self.rotation);
                    let arrow_start_offset = self.translation + zoom_and_rotate * vec2(-0.5, 0.5);

                    // Paints an arrow pointing from bottom-left (-0.5, 0.5) to top-right (0.5, -0.5), but
                    // scaled, rotated, and translated according to the current touch gesture:
                    let arrow_start = Pos2::ZERO + arrow_start_offset;
                    let arrow_direction = zoom_and_rotate * vec2(1., -1.);

                    painter.arrow(to_screen * arrow_start,
                                  to_screen.scale() * arrow_direction,
                                  Stroke::new(stroke_width, color));
                });

*/
/*        CentralPanel::default().show(ctx, |ui| {
            use egui::plot::{Line, Plot, Value, Values};
            let sin = (0..1000).map(|i| {
                let x = i as f64 * 0.01;
                Value::new(x, x.sin())
            });
            let line = Line::new(Values::from_values_iter(sin));
            Plot::new("my_plot").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));

           // let text = Shape::text()
           // ui.add(Shape::text())
        });*/
        CentralPanel::default().show(ctx, |ui| {
            Frame::dark_canvas(ui.style()).show(ui, |ui| {
                let (mut response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());
                let painter_proportions = response.rect.square_proportions();
                let to_screen = RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO - painter_proportions, 2.0 * painter_proportions * self.zoom),
                    response.rect.translate(self.translation)

                );
                if response.dragged_by(PointerButton::Primary) {
                    response = response.on_hover_cursor(CursorIcon::Grabbing);
                    self.translation = self.translation + response.drag_delta();
                }
                if let Some(hover_pos) = response.hover_pos() {
                    self.zoom = self.zoom * ui.input().zoom_delta();
                }
                let rect = to_screen.transform_rect(Rect::from_center_size(Pos2::ZERO, Vec2::new(0.5, 0.6)));
                let color = Color32::DARK_RED;
                let stroke = Stroke::new(1.0,Color32::DARK_GRAY);
                painter.rect(rect, 0.1, color, stroke);
            });
        });
    }

}
