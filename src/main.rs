mod pipeline;
mod image_io;
mod spline;

use eframe::{egui, egui_wgpu};
use pipeline::{Pipeline, ColorSettings};
use image::{DynamicImage, GenericImageView, ImageEncoder};
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
    id: u64,
    pos: f32,
    color: egui::Color32,
}

#[derive(PartialEq, Clone, Copy)]
enum ExportFormat {
    Png,
    Jpg,
    Webp,
}

struct ExportSettings {
    format: ExportFormat,
    compression: f32, // 0.0 to 1.0
    transparency: bool,
    use_percentage: bool,
    percentage: f32,
    width_px: u32,
    height_px: u32,
    link_aspect: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            compression: 0.8,
            transparency: true,
            use_percentage: true,
            percentage: 1.0,
            width_px: 1920,
            height_px: 1080,
            link_aspect: true,
        }
    }
}

#[derive(PartialEq, Clone)]
enum MenuLevel {
    Main,
    AdjustList,
    DitherList,
    SingleAdjustment(&'static str),
    SingleDither(&'static str),
}

#[derive(Clone)]
struct QuickMenuState {
    pos: egui::Pos2,
    level: MenuLevel,
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
    selected_stop_id: Option<u64>,
    next_stop_id: u64,
    curve_points: Vec<egui::Pos2>,
    dragging_point_idx: Option<usize>,
    // UI state
    active_tab: Tab,
    // View state
    zoom_factor: f32,
    fit_to_screen: bool,
    pan_offset: egui::Vec2,
    // Quick Menu
    quick_menu: Option<QuickMenuState>,
    // Export state
    show_export_window: bool,
    export_settings: ExportSettings,
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
            GradientStop { id: 0, pos: 0.0, color: egui::Color32::BLACK },
            GradientStop { id: 1, pos: 1.0, color: egui::Color32::WHITE },
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
            selected_stop_id: Some(0),
            next_stop_id: 2,
            curve_points,
            dragging_point_idx: None,
            active_tab: Tab::Adjust,
            zoom_factor: 1.0,
            fit_to_screen: false,
            pan_offset: egui::Vec2::ZERO,
            quick_menu: None,
            show_export_window: false,
            export_settings: ExportSettings::default(),
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
            GradientStop { id: 0, pos: 0.0, color: egui::Color32::BLACK },
            GradientStop { id: 1, pos: 1.0, color: egui::Color32::WHITE },
        ];
        self.selected_stop_id = Some(0);
        self.next_stop_id = 2;
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
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

        self.current_image = Some(img.clone());
        self.export_settings.width_px = img.width();
        self.export_settings.height_px = img.height();
        self.input_texture = Some(input_tex);
        self.output_texture = Some(output_tex);
        self.egui_texture_id = Some(tex_id);
    }

    fn export_image(&mut self) {
        let (Some(device), Some(queue), Some(input_tex), Some(current_img)) = (&self.device, &self.queue, &self.input_texture, &self.current_image) else { return };
        
        let width = current_img.width();
        let height = current_img.height();
        
        let output_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("export_output_texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.target_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        self.pipeline.render(
            device,
            queue,
            &input_tex.create_view(&wgpu::TextureViewDescriptor::default()),
            &output_tex.create_view(&wgpu::TextureViewDescriptor::default()),
            &self.settings,
        );

        // 1. Staging buffer for readback
        let bytes_per_pixel = 4;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("export_staging_buffer"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("export_encoder") });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &output_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );

        queue.submit(Some(encoder.finish()));

        // 2. Map buffer and read pixels
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(())) = rx.recv() {
            let data = buffer_slice.get_mapped_range();
            let mut pixels = Vec::with_capacity((width * height * 4) as usize);
            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + (width * 4) as usize;
                let row_pixels = &data[start..end];
                
                for chunk in row_pixels.chunks_exact(4) {
                    let r = (chunk[0] as f32 / 255.0).powf(1.0/2.4) * 255.0;
                    let g = (chunk[1] as f32 / 255.0).powf(1.0/2.4) * 255.0;
                    let b = (chunk[2] as f32 / 255.0).powf(1.0/2.4) * 255.0;
                    let a = chunk[3];
                    pixels.push(r as u8);
                    pixels.push(g as u8);
                    pixels.push(b as u8);
                    pixels.push(a);
                }
            }
            drop(data);
            staging_buffer.unmap();

            // 3. Process with image crate
            if let Some(img_buffer) = image::RgbaImage::from_raw(width, height, pixels) {
                let mut dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);
                
                // Final resize to user's requested export dimensions
                if dynamic_img.width() != self.export_settings.width_px || dynamic_img.height() != self.export_settings.height_px {
                    dynamic_img = dynamic_img.resize_exact(self.export_settings.width_px, self.export_settings.height_px, image::imageops::FilterType::Nearest);
                }
                if dynamic_img.width() != self.export_settings.width_px || dynamic_img.height() != self.export_settings.height_px {
                    dynamic_img = dynamic_img.resize_exact(self.export_settings.width_px, self.export_settings.height_px, image::imageops::FilterType::Nearest);
                }

                // Transparency handle
                if !self.export_settings.transparency || self.export_settings.format == ExportFormat::Jpg {
                    let rgb = dynamic_img.to_rgb8();
                    dynamic_img = image::DynamicImage::ImageRgb8(rgb);
                }

                // 4. File dialog and save
                let (ext, filter) = match self.export_settings.format {
                    ExportFormat::Png => ("png", "Portable Network Graphics"),
                    ExportFormat::Jpg => ("jpg", "JPEG"),
                    ExportFormat::Webp => ("webp", "WebP"),
                };

                let dither_names = [
                    "None", "Threshold", "Random", "Bayer", "BlueNoise", 
                    "DiffusionApprox", "Stucki", "Atkinson", "GradientBased", "LatticeBoltzmann"
                ];
                let dither_idx = self.settings.dither_type as usize;
                let dither_name = dither_names.get(dither_idx).unwrap_or(&"Custom");
                
                let color_suffix = if self.settings.grad_enabled > 0.5 { "_Colored" } else { "" };
                let default_name = format!("VibeDither_{}{}.{}", dither_name, color_suffix, ext);

                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(filter, &[ext])
                    .set_file_name(&default_name)
                    .save_file() {
                    
                    match self.export_settings.format {
                        ExportFormat::Png => { 
                            let mut file = std::fs::File::create(&path).unwrap();
                            let level = if self.export_settings.compression > 0.8 {
                                image::codecs::png::CompressionType::Best
                            } else if self.export_settings.compression > 0.3 {
                                image::codecs::png::CompressionType::Default
                            } else {
                                image::codecs::png::CompressionType::Fast
                            };
                            let encoder = image::codecs::png::PngEncoder::new_with_quality(&mut file, level, image::codecs::png::FilterType::Adaptive);
                            let (w, h) = dynamic_img.dimensions();
                            let color_type = dynamic_img.color();
                            encoder.write_image(dynamic_img.as_bytes(), w, h, color_type).ok();
                        },
                        ExportFormat::Jpg => {
                            let mut file = std::fs::File::create(&path).unwrap();
                            let quality = (self.export_settings.compression * 100.0).clamp(1.0, 100.0) as u8;
                            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
                            encoder.encode_image(&dynamic_img).ok();
                        },
                        ExportFormat::Webp => { 
                            dynamic_img.save(path).ok(); 
                        },
                    }
                }
            }
        }
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

                    if ui.button("Export Image").clicked() {
                        self.show_export_window = true;
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
                            let dither_names = [
                                "None", 
                                "Threshold", 
                                "Random", 
                                "Bayer", 
                                "Blue Noise", 
                                "Diffusion Approx", 
                                "Stucki",
                                "Atkinson",
                                "Gradient Based",
                                "Lattice-Boltzmann"
                            ];
                            
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
                                
                                // --- Blender-style Gradient Ramp ---
                                ui.vertical(|ui| {
                                    let ramp_height = 20.0;
                                    let (ramp_rect, _) = ui.allocate_at_least(egui::vec2(ui.available_width(), ramp_height), egui::Sense::hover());
                                    let _rect = ui.allocate_space(egui::vec2(ui.available_width(), 15.0)); // space for handles

                                    // Draw the actual gradient in the bar
                                    for i in 0..255 {
                                        let t0 = i as f32 / 255.0;
                                        let t1 = (i + 1) as f32 / 255.0;
                                        let x0 = ramp_rect.left() + t0 * ramp_rect.width();
                                        let x1 = ramp_rect.left() + t1 * ramp_rect.width();
                                        
                                        let c0 = egui::Color32::from_rgba_unmultiplied(
                                            self.gradient_data[i * 4],
                                            self.gradient_data[i * 4 + 1],
                                            self.gradient_data[i * 4 + 2],
                                            255
                                        );
                                        
                                        let r = egui::Rect::from_min_max(egui::pos2(x0, ramp_rect.top()), egui::pos2(x1, ramp_rect.bottom()));
                                        ui.painter().rect_filled(r, 0.0, c0);
                                    }
                                    ui.painter().rect_stroke(ramp_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)));

                                    // Draw markers (handles)
                                    let mut active_id = self.selected_stop_id;
                                    let mut dragged_id = None;
                                    let mut new_pos_val = 0.0;

                                    for stop in self.gradient_stops.iter() {
                                        let x = ramp_rect.left() + stop.pos * ramp_rect.width();
                                        let y = ramp_rect.bottom() + 2.0;
                                        
                                        let is_selected = Some(stop.id) == active_id;
                                        let color = if is_selected { egui::Color32::WHITE } else { egui::Color32::from_rgb(150, 150, 150) };
                                        
                                        // Draw triangle handle
                                        let p1 = egui::pos2(x, y);
                                        let p2 = egui::pos2(x - 5.0, y + 8.0);
                                        let p3 = egui::pos2(x + 5.0, y + 8.0);
                                        ui.painter().add(egui::Shape::convex_polygon(vec![p1, p2, p3], color, egui::Stroke::NONE));
                                        
                                        // Handle interaction
                                        let handle_rect = egui::Rect::from_center_size(egui::pos2(x, y + 4.0), egui::vec2(10.0, 10.0));
                                        let handle_res = ui.interact(handle_rect, egui::Id::new(("grad", stop.id)), egui::Sense::click_and_drag());
                                        
                                        if handle_res.clicked() {
                                            active_id = Some(stop.id);
                                        }
                                        
                                        if handle_res.dragged() {
                                            active_id = Some(stop.id);
                                            let delta_x = handle_res.drag_delta().x / ramp_rect.width();
                                            new_pos_val = (stop.pos + delta_x).clamp(0.0, 1.0);
                                            dragged_id = Some(stop.id);
                                        }
                                    }

                                    if let Some(id) = dragged_id {
                                        if let Some(stop) = self.gradient_stops.iter_mut().find(|s| s.id == id) {
                                            stop.pos = new_pos_val;
                                            stops_changed = true;
                                        }
                                    }
                                    self.selected_stop_id = active_id;

                                    // Bottom controls for the selected stop
                                    ui.horizontal(|ui| {
                                        if ui.button("+").on_hover_text("Add Stop").clicked() {
                                            let new_id = self.next_stop_id;
                                            self.next_stop_id += 1;
                                            self.gradient_stops.push(GradientStop { id: new_id, pos: 0.5, color: egui::Color32::GRAY });
                                            self.selected_stop_id = Some(new_id);
                                            stops_changed = true;
                                        }
                                        
                                        if let Some(id) = self.selected_stop_id {
                                            if self.gradient_stops.len() > 2 {
                                                if ui.button("-").on_hover_text("Remove Selected").clicked() {
                                                    self.gradient_stops.retain(|s| s.id != id);
                                                    self.selected_stop_id = self.gradient_stops.first().map(|s| s.id);
                                                    stops_changed = true;
                                                }
                                            }
                                            
                                            ui.separator();
                                            
                                            if let Some(stop) = self.gradient_stops.iter_mut().find(|s| s.id == id) {
                                                if ui.color_edit_button_srgba(&mut stop.color).changed() {
                                                    stops_changed = true;
                                                }
                                                ui.add(egui::DragValue::new(&mut stop.pos).speed(0.01).clamp_range(0.0..=1.0));
                                            }
                                        }
                                    });
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
                
                let (rect, response) = ui.allocate_at_least(ui.available_size(), egui::Sense::click_and_drag());

                // --- Handle Zoom ---
                let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll_delta != 0.0 {
                    let zoom_speed = 0.002;
                    let old_zoom = self.zoom_factor;
                    self.zoom_factor = (self.zoom_factor * (1.0 + scroll_delta * zoom_speed)).clamp(0.1, 32.0);
                    self.fit_to_screen = false;
                    
                    // Zoom towards mouse
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        let relative_pos = mouse_pos - rect.center() - self.pan_offset;
                        let zoom_ratio = self.zoom_factor / old_zoom;
                        self.pan_offset -= relative_pos * (zoom_ratio - 1.0);
                    }
                }

                // --- Handle Pan ---
                if response.dragged_by(egui::PointerButton::Primary) {
                    self.pan_offset += response.drag_delta();
                }

                // --- Handle Quick Menu Trigger ---
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.quick_menu = Some(QuickMenuState {
                            pos,
                            level: MenuLevel::Main,
                        });
                    }
                }

                let display_size = if self.fit_to_screen {
                    let ratio = (rect.width() / img_size.x).min(rect.height() / img_size.y);
                    img_size * ratio
                } else {
                    img_size * self.zoom_factor
                };

                let image_rect = egui::Rect::from_center_size(rect.center() + self.pan_offset, display_size);
                
                // Use a clip rect to keep image within panel
                ui.set_clip_rect(rect);
                ui.painter().image(tex_id, image_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Drag and drop an image or use 'Load Image'");
                });
            }
        });

        // --- Export Window ---
        // ... (keep existing export window code)

        // --- Quick Access Menu ---
        if let Some(menu) = self.quick_menu.clone() {
            let mut close_menu = false;
            let mut menu_changed = false;

            egui::Area::new(egui::Id::new("quick_menu"))
                .fixed_pos(menu.pos)
                .show(ctx, |ui| {
                    let frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(0, 0, 0))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 65)))
                        .inner_margin(4.0);

                    frame.show(ui, |ui| {
                        ui.set_max_width(200.0);
                        match menu.level {
                            MenuLevel::Main => {
                                if ui.button("> ADJUST").clicked() {
                                    self.quick_menu.as_mut().unwrap().level = MenuLevel::AdjustList;
                                }
                                if ui.button("> DITHER").clicked() {
                                    self.quick_menu.as_mut().unwrap().level = MenuLevel::DitherList;
                                }
                                if ui.button("> CLOSE").clicked() {
                                    close_menu = true;
                                }
                            }
                            MenuLevel::AdjustList => {
                                let adjs = [
                                    ("Exposure", "exposure"), ("Contrast", "contrast"), 
                                    ("Highlights", "highlights"), ("Shadows", "shadows"),
                                    ("Whites", "whites"), ("Blacks", "blacks"),
                                    ("Temp", "temperature"), ("Tint", "tint"),
                                    ("Vibrance", "vibrance"), ("Saturation", "saturation"),
                                    ("Sharpness", "sharpness")
                                ];
                                for (label, id) in adjs {
                                    if ui.button(label).clicked() {
                                        self.quick_menu.as_mut().unwrap().level = MenuLevel::SingleAdjustment(id);
                                    }
                                }
                                if ui.button("< BACK").clicked() {
                                    self.quick_menu.as_mut().unwrap().level = MenuLevel::Main;
                                }
                            }
                            MenuLevel::DitherList => {
                                let diths = [
                                    ("Pixel Scale", "scale"), ("Threshold", "threshold"),
                                    ("Posterize", "posterize"), ("Matrix Size", "bayer"),
                                    ("Color Mode", "color")
                                ];
                                for (label, id) in diths {
                                    if ui.button(label).clicked() {
                                        self.quick_menu.as_mut().unwrap().level = MenuLevel::SingleDither(id);
                                    }
                                }
                                if ui.button("< BACK").clicked() {
                                    self.quick_menu.as_mut().unwrap().level = MenuLevel::Main;
                                }
                            }
                            MenuLevel::SingleAdjustment(id) => {
                                match id {
                                    "exposure" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.exposure, -5.0..=5.0).text("Exp")).changed(),
                                    "contrast" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.contrast, 0.0..=2.0).text("Con")).changed(),
                                    "highlights" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.highlights, -1.0..=1.0).text("High")).changed(),
                                    "shadows" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.shadows, -1.0..=1.0).text("Shad")).changed(),
                                    "whites" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.whites, -1.0..=1.0).text("White")).changed(),
                                    "blacks" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.blacks, -1.0..=1.0).text("Black")).changed(),
                                    "temperature" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.temperature, -1.0..=1.0).text("Temp")).changed(),
                                    "tint" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.tint, -1.0..=1.0).text("Tint")).changed(),
                                    "vibrance" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.vibrance, -1.0..=1.0).text("Vib")).changed(),
                                    "saturation" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.saturation, 0.0..=2.0).text("Sat")).changed(),
                                    "sharpness" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.sharpness, 0.0..=2.0).text("Sharp")).changed(),
                                    _ => {}
                                }
                                if ui.button("OK").clicked() {
                                    self.quick_menu.as_mut().unwrap().level = MenuLevel::AdjustList;
                                }
                            }
                            MenuLevel::SingleDither(id) => {
                                match id {
                                    "scale" => {
                                        let mut s = self.settings.dither_scale as i32;
                                        if ui.add(egui::Slider::new(&mut s, 1..=32).text("Scale")).changed() {
                                            self.settings.dither_scale = s as f32;
                                            menu_changed = true;
                                        }
                                    }
                                    "threshold" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.dither_threshold, 0.0..=1.0).text("Thresh")).changed(),
                                    "posterize" => menu_changed |= ui.add(egui::Slider::new(&mut self.settings.posterize_levels, 0.0..=64.0).text("Post")).changed(),
                                    "bayer" => {
                                        ui.horizontal(|ui| {
                                            let sizes = [2, 3, 4, 8];
                                            for s in sizes {
                                                if ui.selectable_label(self.settings.bayer_size as i32 == s, format!("{}", s)).clicked() {
                                                    self.settings.bayer_size = s as f32;
                                                    menu_changed = true;
                                                }
                                            }
                                        });
                                    }
                                    "color" => {
                                        let mut c = self.settings.dither_color > 0.5;
                                        if ui.checkbox(&mut c, "Color").changed() {
                                            self.settings.dither_color = if c { 1.0 } else { 0.0 };
                                            menu_changed = true;
                                        }
                                    }
                                    _ => {}
                                }
                                if ui.button("OK").clicked() {
                                    self.quick_menu.as_mut().unwrap().level = MenuLevel::DitherList;
                                }
                            }
                        }
                    });
                });

            if close_menu || (ctx.input(|i| i.pointer.any_pressed()) && !ctx.is_using_pointer()) {
                self.quick_menu = None;
            }

            if menu_changed && self.egui_texture_id.is_some() {
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
        }
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