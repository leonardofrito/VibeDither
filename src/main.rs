mod pipeline;
mod image_io;
mod spline;

use eframe::{egui, egui_wgpu};
use pipeline::{Pipeline, ColorSettings};
use image::DynamicImage;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    env_logger::init(); 

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_drag_and_drop(true),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "VibeDither",
        options,
        Box::new(|cc| {
            setup_custom_style(&cc.egui_ctx);
            Box::new(VibeDitherApp::new(cc))
        }),
    )
}

#[derive(PartialEq)]
enum Tab {
    Adjust,
    Dither,
}

#[derive(Clone, Copy)]
struct GradientStop {
    pos: f32,
    color: egui::Color32,
}

struct VibeDitherApp {
    pipeline: Pipeline,
    current_image: Option<DynamicImage>,
    // GPU-side resources
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
    renderer: Option<Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>>,
    target_format: wgpu::TextureFormat,
    input_texture: Option<wgpu::Texture>,
    output_texture: Option<wgpu::Texture>,
    egui_texture_id: Option<egui::TextureId>,
    settings: ColorSettings,
    curves_data: [u8; 1024], // 256 * 4 (RGBA)
    gradient_data: [u8; 1024], // 256 * 4 (RGBA)
    gradient_stops: Vec<GradientStop>,
    curve_points: Vec<egui::Pos2>,
    dragging_point_idx: Option<usize>,
    // UI state
    active_tab: Tab,
    // Zoom state
    zoom_factor: f32,
    fit_to_screen: bool,
}

impl VibeDitherApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut pipeline = Pipeline::new();
        let mut device = None;
        let mut queue = None;
        let mut renderer = None;
        let mut target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        
        if let Some(wgpu_render_state) = &cc.wgpu_render_state {
            device = Some(wgpu_render_state.device.clone());
            queue = Some(wgpu_render_state.queue.clone());
            renderer = Some(wgpu_render_state.renderer.clone());
            target_format = wgpu_render_state.target_format;
            pipeline.init(&wgpu_render_state.device, target_format);
        }

        let mut curves_data = [0u8; 1024];
        let curve_points = vec![egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)];
        let lut = spline::interpolate_spline(&curve_points);
        for i in 0..256 {
            curves_data[i * 4] = lut[i];
            curves_data[i * 4 + 1] = lut[i];
            curves_data[i * 4 + 2] = lut[i];
            curves_data[i * 4 + 3] = 255;
        }

        let gradient_stops = vec![
            GradientStop { pos: 0.0, color: egui::Color32::BLACK },
            GradientStop { pos: 1.0, color: egui::Color32::WHITE },
        ];
        let mut gradient_data = [0u8; 1024];
        Self::generate_gradient_data(&gradient_stops, &mut gradient_data);

        Self {
            pipeline,
            current_image: None,
            device,
            queue,
            renderer,
            target_format,
            input_texture: None,
            output_texture: None,
            egui_texture_id: None,
            settings: ColorSettings::default(),
            curves_data,
            gradient_data,
            gradient_stops,
            curve_points,
            dragging_point_idx: None,
            active_tab: Tab::Adjust,
            zoom_factor: 1.0,
            fit_to_screen: true,
        }
    }

    fn generate_gradient_data(stops: &[GradientStop], data: &mut [u8; 1024]) {
        for i in 0..256 {
            let t = i as f32 / 255.0;
            
            // Find surrounding stops
            let mut lower = &stops[0];
            let mut upper = &stops[stops.len() - 1];
            
            for stop in stops {
                if stop.pos <= t && stop.pos >= lower.pos {
                    lower = stop;
                }
                if stop.pos >= t && stop.pos <= upper.pos {
                    upper = stop;
                }
            }
            
            let color = if (upper.pos - lower.pos).abs() < 0.0001 {
                lower.color
            } else {
                let factor = (t - lower.pos) / (upper.pos - lower.pos);
                egui::Color32::from_rgba_unmultiplied(
                    (lower.color.r() as f32 * (1.0 - factor) + upper.color.r() as f32 * factor) as u8,
                    (lower.color.g() as f32 * (1.0 - factor) + upper.color.g() as f32 * factor) as u8,
                    (lower.color.b() as f32 * (1.0 - factor) + upper.color.b() as f32 * factor) as u8,
                    255,
                )
            };
            
            data[i * 4] = color.r();
            data[i * 4 + 1] = color.g();
            data[i * 4 + 2] = color.b();
            data[i * 4 + 3] = 255;
        }
    }

    fn reset_adjustments(&mut self) {
        self.settings = ColorSettings::default();
        self.curve_points = vec![egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)];
        let lut = spline::interpolate_spline(&self.curve_points);
        for i in 0..256 {
            self.curves_data[i * 4] = lut[i];
            self.curves_data[i * 4 + 1] = lut[i];
            self.curves_data[i * 4 + 2] = lut[i];
            self.curves_data[i * 4 + 3] = 255;
        }

        self.gradient_stops = vec![
            GradientStop { pos: 0.0, color: egui::Color32::BLACK },
            GradientStop { pos: 1.0, color: egui::Color32::WHITE },
        ];
        Self::generate_gradient_data(&self.gradient_stops, &mut self.gradient_data);

        if let Some(queue) = &self.queue {
            self.pipeline.update_curves(queue, &self.curves_data);
            self.pipeline.update_gradient(queue, &self.gradient_data);
        }
    }

    fn load_image_to_gpu(&mut self, _ctx: &egui::Context, img: DynamicImage) {
        self.reset_adjustments();
        let Some(device) = &self.device else { return };
        let Some(queue) = &self.queue else { return };
        let Some(renderer) = &self.renderer else { return };

        let input_tex = self.pipeline.create_texture_from_image(device, queue, &img);
        
        let output_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("output_texture"),
            size: input_tex.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.target_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let tex_id = renderer.write().register_native_texture(
            device,
            &output_tex.create_view(&wgpu::TextureViewDescriptor::default()),
            wgpu::FilterMode::Nearest,
        );

        // Upload current curves and gradient
        self.pipeline.update_curves(queue, &self.curves_data);
        self.pipeline.update_gradient(queue, &self.gradient_data);

        self.current_image = Some(img);
        self.input_texture = Some(input_tex);
        self.output_texture = Some(output_tex);
        self.egui_texture_id = Some(tex_id);
    }
}

impl eframe::App for VibeDitherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle drag and drop
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(path) = i.raw.dropped_files[0].path.as_ref() {
                    if let Ok(img) = image_io::load_from_path(path) {
                        self.load_image_to_gpu(ctx, img);
                    }
                }
            }
        });

        egui::SidePanel::left("control_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("VibeDither v0.1");
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label("IMAGE CONTROLS");
                    
                    if ui.button("Load Image").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                            .pick_file() {
                            if let Ok(img) = image_io::load_from_path(&path) {
                                self.load_image_to_gpu(ctx, img);
                            }
                        }
                    }

                    if ui.button("Paste from Clipboard").clicked() {
                        if let Some(img) = image_io::get_clipboard_image() {
                            self.load_image_to_gpu(ctx, img);
                        }
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.active_tab, Tab::Adjust, "ADJUST");
                        ui.selectable_value(&mut self.active_tab, Tab::Dither, "DITHER");
                    });
                    
                    ui.separator();

                    let mut changed = false;

                    match self.active_tab {
                        Tab::Adjust => {
                            ui.label("BASIC ADJUSTMENTS");
                            
                            ui.group(|ui| {
                                ui.label("Light");
                                changed |= ui.add(egui::Slider::new(&mut self.settings.exposure, -5.0..=5.0).text("Exposure")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.contrast, 0.0..=2.0).text("Contrast")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.highlights, -1.0..=1.0).text("Highlights")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.shadows, -1.0..=1.0).text("Shadows")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.whites, -1.0..=1.0).text("Whites")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.blacks, -1.0..=1.0).text("Blacks")).changed();
                            });

                            ui.group(|ui| {
                                ui.label("Color");
                                changed |= ui.add(egui::Slider::new(&mut self.settings.temperature, -1.0..=1.0).text("Temperature")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.tint, -1.0..=1.0).text("Tint")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.vibrance, -1.0..=1.0).text("Vibrance")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.saturation, 0.0..=2.0).text("Saturation")).changed();
                            });

                            ui.group(|ui| {
                                ui.label("Detail");
                                changed |= ui.add(egui::Slider::new(&mut self.settings.sharpness, 0.0..=2.0).text("Sharpness")).changed();
                            });

                            if ui.button("Reset Adjustments").clicked() {
                                self.settings = ColorSettings::default();
                                changed = true;
                            }

                            ui.separator();
                            ui.label("RGB CURVES");
                            
                            let mut curves_changed = false;
                            ui.vertical(|ui| {
                                let size = egui::vec2(ui.available_width(), 150.0);
                                let (rect, response) = ui.allocate_at_least(size, egui::Sense::click_and_drag());
                                
                                ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(15, 15, 15));
                                ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40)));
                                
                                for i in 1..4 {
                                    let x = rect.left() + rect.width() * (i as f32 / 4.0);
                                    let y = rect.top() + rect.height() * (i as f32 / 4.0);
                                    ui.painter().line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(0.5, egui::Color32::from_rgb(30, 30, 30)));
                                    ui.painter().line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(0.5, egui::Color32::from_rgb(30, 30, 30)));
                                }

                                let lut = spline::interpolate_spline(&self.curve_points);
                                let mut points: Vec<egui::Pos2> = Vec::new();
                                for i in 0..256 {
                                    let val = lut[i];
                                    let x = rect.left() + (i as f32 / 255.0) * rect.width();
                                    let y = rect.bottom() - (val as f32 / 255.0) * rect.height();
                                    points.push(egui::pos2(x, y));
                                }
                                ui.painter().add(egui::Shape::line(points, egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 65))));

                                if let Some(pos) = response.interact_pointer_pos() {
                                    let x_norm = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                                    let y_norm = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
                                    
                                    if response.drag_started() {
                                        let mut closest_idx = None;
                                        let mut min_dist = 0.05;
                                        for (idx, p) in self.curve_points.iter().enumerate() {
                                            let dist = (p.x - x_norm).abs();
                                            if dist < min_dist {
                                                min_dist = dist;
                                                closest_idx = Some(idx);
                                            }
                                        }
                                        self.dragging_point_idx = closest_idx;
                                    }

                                    if response.dragged() {
                                        if let Some(idx) = self.dragging_point_idx {
                                            if idx == 0 {
                                                self.curve_points[idx].y = y_norm;
                                            } else if idx == self.curve_points.len() - 1 {
                                                self.curve_points[idx].y = y_norm;
                                            } else {
                                                let min_x = self.curve_points[idx-1].x + 0.001;
                                                let max_x = self.curve_points[idx+1].x - 0.001;
                                                self.curve_points[idx] = egui::pos2(x_norm.clamp(min_x, max_x), y_norm);
                                            }
                                            curves_changed = true;
                                        }
                                    }

                                    if response.drag_stopped() {
                                        self.dragging_point_idx = None;
                                    }

                                    if response.clicked() && !response.dragged() {
                                        let mut exists = false;
                                        for p in &self.curve_points {
                                            if (p.x - x_norm).abs() < 0.02 { exists = true; break; }
                                        }
                                        if !exists {
                                            self.curve_points.push(egui::pos2(x_norm, y_norm));
                                            self.curve_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
                                            curves_changed = true;
                                        }
                                    }
                                }

                                if response.secondary_clicked() {
                                    if let Some(pos) = response.interact_pointer_pos() {
                                        let x_norm = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                                        let mut to_remove = None;
                                        for (idx, p) in self.curve_points.iter().enumerate() {
                                            if (p.x - x_norm).abs() < 0.03 && idx != 0 && idx != self.curve_points.len() - 1 {
                                                to_remove = Some(idx);
                                                break;
                                            }
                                        }
                                        if let Some(idx) = to_remove {
                                            self.curve_points.remove(idx);
                                            curves_changed = true;
                                        }
                                    }
                                }

                                for p in &self.curve_points {
                                    let px = rect.left() + p.x * rect.width();
                                    let py = rect.bottom() - p.y * rect.height();
                                    ui.painter().circle_filled(egui::pos2(px, py), 3.0, egui::Color32::WHITE);
                                }
                                
                                if ui.button("Reset Curves").clicked() {
                                    self.curve_points = vec![egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)];
                                    curves_changed = true;
                                }
                            });

                            if curves_changed {
                                let lut = spline::interpolate_spline(&self.curve_points);
                                for i in 0..256 {
                                    self.curves_data[i * 4] = lut[i];
                                    self.curves_data[i * 4 + 1] = lut[i];
                                    self.curves_data[i * 4 + 2] = lut[i];
                                }
                                if let Some(queue) = &self.queue {
                                    self.pipeline.update_curves(queue, &self.curves_data);
                                }
                                changed = true;
                            }
                        },
                        Tab::Dither => {
                            ui.label("DITHERING CONTROLS");
                            
                            let dither_type = self.settings.dither_type as usize;
                            let dither_names = ["None", "Threshold", "Random", "Bayer", "Blue Noise", "Error Diffusion"];
                            
                            egui::ComboBox::from_label("Algorithm")
                                .selected_text(dither_names[dither_type.min(dither_names.len() - 1)])
                                .show_ui(ui, |ui| {
                                    for (i, name) in dither_names.iter().enumerate() {
                                        if ui.selectable_label(dither_type == i, *name).clicked() {
                                            self.settings.dither_type = i as f32;
                                            self.settings.dither_enabled = if i > 0 { 1.0 } else { 0.0 };
                                            // Reset mode-specific dither color toggle
                                            self.settings.dither_color = 0.0;
                                            changed = true;
                                        }
                                    }
                                });

                            ui.add_enabled_ui(dither_type > 0, |ui| {
                                let mut scale_int = self.settings.dither_scale as i32;
                                if ui.add(egui::Slider::new(&mut scale_int, 1..=32).text("Pixel Scale")).changed() {
                                    self.settings.dither_scale = scale_int as f32;
                                    changed = true;
                                }

                                if dither_type == 1 {
                                    changed |= ui.add(egui::Slider::new(&mut self.settings.dither_threshold, 0.0..=1.0).text("Threshold")).changed();
                                }
                                
                                // Posterize as a dither effect
                                ui.group(|ui| {
                                    ui.label("Posterize");
                                    let mut use_posterize = self.settings.posterize_levels > 0.0;
                                    if ui.checkbox(&mut use_posterize, "Enabled").changed() {
                                        self.settings.posterize_levels = if use_posterize { 4.0 } else { 0.0 };
                                        changed = true;
                                    }
                                    ui.add_enabled_ui(use_posterize, |ui| {
                                        changed |= ui.add(egui::Slider::new(&mut self.settings.posterize_levels, 2.0..=64.0).text("Levels")).changed();
                                    });
                                });
                                
                                if dither_type == 3 { // Bayer
                                    ui.horizontal(|ui| {
                                        ui.label("Matrix Size:");
                                        let sizes = [2, 3, 4, 8];
                                        for s in sizes {
                                            if ui.selectable_label(self.settings.bayer_size as i32 == s, format!("{}x{}", s, s)).clicked() {
                                                self.settings.bayer_size = s as f32;
                                                changed = true;
                                            }
                                        }
                                    });
                                }

                                if dither_type >= 1 { // Threshold, Random, Bayer, Blue, Error Diff
                                    let mut color_dither = self.settings.dither_color > 0.5;
                                    if ui.checkbox(&mut color_dither, "Color Dithering").changed() {
                                        self.settings.dither_color = if color_dither { 1.0 } else { 0.0 };
                                        changed = true;
                                    }
                                }
                            });

                            ui.separator();
                            ui.label("GRADIENT REMAP");
                            let mut grad_enabled = self.settings.grad_enabled > 0.5;
                            if ui.checkbox(&mut grad_enabled, "Enable Gradient Remap").changed() {
                                self.settings.grad_enabled = if grad_enabled { 1.0 } else { 0.0 };
                                changed = true;
                            }

                            ui.add_enabled_ui(grad_enabled, |ui| {
                                let mut stops_changed = false;
                                
                                ui.vertical(|ui| {
                                    let mut to_remove = None;
                                    let stops_len = self.gradient_stops.len();
                                    for (idx, stop) in self.gradient_stops.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            if ui.color_edit_button_srgba(&mut stop.color).changed() {
                                                stops_changed = true;
                                            }
                                            let mut pos = stop.pos;
                                            if ui.add(egui::Slider::new(&mut pos, 0.0..=1.0).show_value(false)).changed() {
                                                stop.pos = pos;
                                                stops_changed = true;
                                            }
                                            if idx != 0 && idx != stops_len - 1 {
                                                if ui.button("x").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            } else {
                                                ui.add_enabled(false, egui::Button::new(" "));
                                            }
                                        });
                                    }
                                    
                                    if let Some(idx) = to_remove {
                                        self.gradient_stops.remove(idx);
                                        stops_changed = true;
                                    }
                                    
                                    if ui.button("+ Add Stop").clicked() {
                                        self.gradient_stops.push(GradientStop { pos: 0.5, color: egui::Color32::GRAY });
                                        stops_changed = true;
                                    }
                                });

                                if stops_changed {
                                    self.gradient_stops.sort_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap());
                                    Self::generate_gradient_data(&self.gradient_stops, &mut self.gradient_data);
                                    if let Some(queue) = &self.queue {
                                        self.pipeline.update_gradient(queue, &self.gradient_data);
                                    }
                                    changed = true;
                                }
                            });
                        }
                    }

                    ui.separator();

                    if (changed || self.current_image.is_some()) && self.egui_texture_id.is_some() {
                        if let (Some(device), Some(queue), Some(input), Some(output)) = (&self.device, &self.queue, &self.input_texture, &self.output_texture) {
                            self.pipeline.render(
                                device,
                                queue,
                                &input.create_view(&wgpu::TextureViewDescriptor::default()),
                                &output.create_view(&wgpu::TextureViewDescriptor::default()),
                                &self.settings,
                            );
                        }
                    }
                });
            });

        egui::TopBottomPanel::bottom("zoom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Zoom: {:.0}%", self.zoom_factor * 100.0));
                
                egui::ComboBox::from_id_source("zoom_selector")
                    .selected_text(format!("{:.0}%", self.zoom_factor * 100.0))
                    .show_ui(ui, |ui| {
                        let zooms = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0];
                        for z in zooms {
                            if ui.selectable_value(&mut self.zoom_factor, z, format!("{:.0}%", z * 100.0)).clicked() {
                                self.fit_to_screen = false;
                            }
                        }
                    });

                if ui.button("100%").clicked() {
                    self.zoom_factor = 1.0;
                    self.fit_to_screen = false;
                }
                
                if ui.button("Fit to Screen").clicked() {
                    self.fit_to_screen = true;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex_id) = self.egui_texture_id {
                let img_size = self.current_image.as_ref().map(|img| egui::vec2(img.width() as f32, img.height() as f32)).unwrap_or(egui::Vec2::ZERO);
                
                let display_size = if self.fit_to_screen {
                    let available = ui.available_size();
                    let ratio = (available.x / img_size.x).min(available.y / img_size.y);
                    img_size * ratio
                } else {
                    img_size * self.zoom_factor
                };

                egui::ScrollArea::both().show(ui, |ui| {
                    ui.image(egui::load::SizedTexture::new(tex_id, display_size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Drag and drop an image or use 'Load Image'");
                });
            }
        });
    }
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style: egui::Style = (*ctx.style()).clone();
    
    let matrix_green = egui::Color32::from_rgb(0, 255, 65);
    let black = egui::Color32::from_rgb(10, 10, 10);

    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(matrix_green);
    style.visuals.widgets.noninteractive.bg_fill = black;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, matrix_green);
    
    style.visuals.widgets.inactive.bg_fill = black;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, matrix_green);
    
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(20, 20, 20);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, matrix_green);
    
    style.visuals.widgets.active.bg_fill = matrix_green;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, black);

    style.visuals.selection.bg_fill = matrix_green.linear_multiply(0.3);

    ctx.set_style(style);
}