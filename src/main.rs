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
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]).with_drag_and_drop(true),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native("VibeDither", options, Box::new(|cc| { setup_custom_style(&cc.egui_ctx); Box::new(VibeDitherApp::new(cc)) }))
}

#[derive(PartialEq)]
enum Tab { Adjust, Dither }

#[derive(Clone, Copy)]
struct GradientStop { id: u64, pos: f32, color: egui::Color32 }

#[derive(PartialEq, Clone, Copy)]
enum ExportFormat { Png, Jpg, Webp }

struct ExportSettings {
    format: ExportFormat, compression: f32, transparency: bool,
    use_percentage: bool, percentage: f32, width_px: u32, height_px: u32, link_aspect: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self { format: ExportFormat::Png, compression: 0.8, transparency: true, use_percentage: true, percentage: 1.0, width_px: 1920, height_px: 1080, link_aspect: true }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum KeyboardFocus { Main, Adjust, Light, Color, Dither, Editing(&'static str), ModeSelection, PosterizeMenu, BayerSizeMenu, GradientMapMenu, GradientPointEdit }

struct VibeDitherApp {
    pipeline: Pipeline, current_image: Option<DynamicImage>,
    device: Option<Arc<wgpu::Device>>, queue: Option<Arc<wgpu::Queue>>, renderer: Option<Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>>,
    target_format: wgpu::TextureFormat, input_texture: Option<wgpu::Texture>, output_texture: Option<wgpu::Texture>,
    egui_texture_id: Option<egui::TextureId>, settings: ColorSettings,
    curves_data: [u8; 1024], gradient_data: [u8; 1024], gradient_stops: Vec<GradientStop>,
    selected_stop_id: Option<u64>, next_stop_id: u64, curve_points: Vec<egui::Pos2>,
    dragging_point_idx: Option<usize>, active_tab: Tab, zoom_factor: f32, fit_to_screen: bool, pan_offset: egui::Vec2,
    focus: KeyboardFocus, last_edit_time: f64, show_export_window: bool, export_settings: ExportSettings,
}

impl VibeDitherApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut pipeline = Pipeline::new();
        let mut device = None; let mut queue = None; let mut renderer = None;
        let mut target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        if let Some(wgpu_render_state) = &cc.wgpu_render_state {
            device = Some(wgpu_render_state.device.clone()); queue = Some(wgpu_render_state.queue.clone()); renderer = Some(wgpu_render_state.renderer.clone());
            target_format = wgpu_render_state.target_format; pipeline.init(&wgpu_render_state.device, target_format);
        }
        let mut curves_data = [0u8; 1024];
        let curve_points = vec![egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)];
        let lut = spline::interpolate_spline(&curve_points);
        for i in 0..256 { curves_data[i * 4] = lut[i]; curves_data[i * 4 + 1] = lut[i]; curves_data[i * 4 + 2] = lut[i]; curves_data[i * 4 + 3] = 255; }
        let gradient_stops = vec![GradientStop { id: 0, pos: 0.0, color: egui::Color32::BLACK }, GradientStop { id: 1, pos: 1.0, color: egui::Color32::WHITE }];
        let mut gradient_data = [0u8; 1024]; Self::generate_gradient_data(&gradient_stops, &mut gradient_data);
        Self {
            pipeline, current_image: None, device, queue, renderer, target_format, input_texture: None, output_texture: None, egui_texture_id: None,
            settings: ColorSettings::default(), curves_data, gradient_data, gradient_stops, selected_stop_id: Some(0), next_stop_id: 2, curve_points, dragging_point_idx: None,
            active_tab: Tab::Adjust, zoom_factor: 1.0, fit_to_screen: false, pan_offset: egui::Vec2::ZERO, focus: KeyboardFocus::Main, last_edit_time: 0.0, show_export_window: false, export_settings: ExportSettings::default(),
        }
    }

    fn generate_gradient_data(stops: &[GradientStop], data: &mut [u8; 1024]) {
        if stops.is_empty() { return; }
        for i in 0..256 {
            let t = i as f32 / 255.0;
            let mut lower = &stops[0]; let mut upper = &stops[stops.len() - 1];
            for stop in stops { if stop.pos <= t && stop.pos >= lower.pos { lower = stop; } if stop.pos >= t && stop.pos <= upper.pos { upper = stop; } }
            let color = if (upper.pos - lower.pos).abs() < 0.0001 { lower.color } else {
                let f = (t - lower.pos) / (upper.pos - lower.pos);
                egui::Color32::from_rgba_unmultiplied(
                    (lower.color.r() as f32 * (1.0 - f) + upper.color.r() as f32 * f) as u8,
                    (lower.color.g() as f32 * (1.0 - f) + upper.color.g() as f32 * f) as u8,
                    (lower.color.b() as f32 * (1.0 - f) + upper.color.b() as f32 * f) as u8, 255,
                )
            };
            data[i * 4] = color.r(); data[i * 4 + 1] = color.g(); data[i * 4 + 2] = color.b(); data[i * 4 + 3] = 255;
        }
    }

    fn reset_adjustments(&mut self) {
        self.settings = ColorSettings::default(); self.curve_points = vec![egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)];
        let lut = spline::interpolate_spline(&self.curve_points);
        for i in 0..256 { self.curves_data[i * 4] = lut[i]; self.curves_data[i * 4 + 1] = lut[i]; self.curves_data[i * 4 + 2] = lut[i]; self.curves_data[i * 4 + 3] = 255; }
        self.gradient_stops = vec![GradientStop { id: 0, pos: 0.0, color: egui::Color32::BLACK }, GradientStop { id: 1, pos: 1.0, color: egui::Color32::WHITE }];
        self.selected_stop_id = Some(0); self.next_stop_id = 2; Self::generate_gradient_data(&self.gradient_stops, &mut self.gradient_data);
        if let Some(q) = &self.queue { self.pipeline.update_curves(q, &self.curves_data); self.pipeline.update_gradient(q, &self.gradient_data); }
    }

    fn load_image_to_gpu(&mut self, _ctx: &egui::Context, img: DynamicImage) {
        self.reset_adjustments(); let Some(device) = &self.device else { return }; let Some(queue) = &self.queue else { return }; let Some(renderer) = &self.renderer else { return };
        let input_tex = self.pipeline.create_texture_from_image(device, queue, &img);
        let output_tex = device.create_texture(&wgpu::TextureDescriptor { label: Some("output_texture"), size: input_tex.size(), mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: self.target_format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[] });
        let tex_id = renderer.write().register_native_texture(device, &output_tex.create_view(&wgpu::TextureViewDescriptor::default()), wgpu::FilterMode::Nearest);
        self.pipeline.update_curves(queue, &self.curves_data); self.pipeline.update_gradient(queue, &self.gradient_data);
        self.current_image = Some(img.clone()); self.export_settings.width_px = img.width(); self.export_settings.height_px = img.height();
                self.output_texture = Some(output_tex);
                self.egui_texture_id = Some(tex_id);
        
                // Immediate render so the image appears instantly
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

    fn export_image(&mut self) {
        let (Some(device), Some(queue), Some(input_tex), Some(current_img)) = (&self.device, &self.queue, &self.input_texture, &self.current_image) else { return };
        let width = current_img.width(); let height = current_img.height();
        let output_tex = device.create_texture(&wgpu::TextureDescriptor { label: Some("export_output_texture"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: self.target_format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[] });
        self.pipeline.render(device, queue, &input_tex.create_view(&wgpu::TextureViewDescriptor::default()), &output_tex.create_view(&wgpu::TextureViewDescriptor::default()), &self.settings);
        let bytes_per_pixel = 4; let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT; let unpadded = width * bytes_per_pixel; let padded = unpadded + (align - unpadded % align) % align;
        let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("export_staging"), size: (padded * height) as u64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("export_enc") });
        encoder.copy_texture_to_buffer(wgpu::ImageCopyTexture { texture: &output_tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, wgpu::ImageCopyBuffer { buffer: &staging, layout: wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(height) } }, wgpu::Extent3d { width, height, depth_or_array_layers: 1 });
        queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..); let (tx, rx) = std::sync::mpsc::channel(); slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap()); device.poll(wgpu::Maintain::Wait);
        if let Ok(Ok(())) = rx.recv() {
            let data = slice.get_mapped_range(); let mut pixels = Vec::with_capacity((width * height * 4) as usize);
            for row in 0..height { let start = (row * padded) as usize; for chunk in data[start..start+(width*4) as usize].chunks_exact(4) { pixels.push(((chunk[0] as f32 / 255.0).powf(1.0/2.4) * 255.0) as u8); pixels.push(((chunk[1] as f32 / 255.0).powf(1.0/2.4) * 255.0) as u8); pixels.push(((chunk[2] as f32 / 255.0).powf(1.0/2.4) * 255.0) as u8); pixels.push(chunk[3]); } }
            drop(data); staging.unmap();
            if let Some(img_buf) = image::RgbaImage::from_raw(width, height, pixels) {
                let mut dimg = image::DynamicImage::ImageRgba8(img_buf);
                if dimg.width() != self.export_settings.width_px || dimg.height() != self.export_settings.height_px { dimg = dimg.resize_exact(self.export_settings.width_px, self.export_settings.height_px, image::imageops::FilterType::Nearest); }
                if !self.export_settings.transparency || self.export_settings.format == ExportFormat::Jpg { dimg = image::DynamicImage::ImageRgb8(dimg.to_rgb8()); }
                let (ext, filt) = match self.export_settings.format { ExportFormat::Png => ("png", "PNG"), ExportFormat::Jpg => ("jpg", "JPEG"), ExportFormat::Webp => ("webp", "WebP") };
                let d_names = ["None", "Threshold", "Random", "Bayer", "BlueNoise", "DiffusionApprox", "Stucki", "Atkinson", "GradientBased", "LatticeBoltzmann"];
                let d_name = d_names.get(self.settings.dither_type as usize).unwrap_or(&"Custom");
                let color_suffix = if self.settings.grad_enabled > 0.5 { "_Colored" } else { "" };
                if let Some(path) = rfd::FileDialog::new().add_filter(filt, &[ext]).set_file_name(&format!("VibeDither_{}{}.{}", d_name, color_suffix, ext)).save_file() {
                    match self.export_settings.format {
                        ExportFormat::Png => {
                            let mut f = std::fs::File::create(&path).unwrap();
                            let level = if self.export_settings.compression > 0.8 { image::codecs::png::CompressionType::Best } else if self.export_settings.compression > 0.3 { image::codecs::png::CompressionType::Default } else { image::codecs::png::CompressionType::Fast };
                            let encoder = image::codecs::png::PngEncoder::new_with_quality(&mut f, level, image::codecs::png::FilterType::Adaptive);
                            let (w, h) = dimg.dimensions(); let color_type = dimg.color();
                            encoder.write_image(dimg.as_bytes(), w, h, color_type).ok();
                        },
                        ExportFormat::Jpg => {
                            let mut f = std::fs::File::create(&path).unwrap();
                            let quality = (self.export_settings.compression * 100.0).clamp(1.0, 100.0) as u8;
                            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut f, quality);
                            encoder.encode_image(&dimg).ok();
                        },
                        ExportFormat::Webp => { dimg.save(path).ok(); },
                    }
                }
            }
        }
    }
}

impl eframe::App for VibeDitherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut changed = false;
        let (esc, space, k_a, k_d, k_q, k_e, k_c, k_h, k_z, k_s, k_b, k_w, k_f, k_t, k_v, k_m, k_o, k_p, k_n, k_g, k_r, k_y, k_l, k_j, k_k, k_up, k_down, k_left, k_right, shift, ctrl, keys_0_9) = ctx.input(|i| (
            i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::Space), i.key_pressed(egui::Key::A), i.key_pressed(egui::Key::D), i.key_pressed(egui::Key::Q), i.key_pressed(egui::Key::E), i.key_pressed(egui::Key::C), i.key_pressed(egui::Key::H), i.key_pressed(egui::Key::Z), i.key_pressed(egui::Key::S), i.key_pressed(egui::Key::B), i.key_pressed(egui::Key::W), i.key_pressed(egui::Key::F), i.key_pressed(egui::Key::T), i.key_pressed(egui::Key::V), i.key_pressed(egui::Key::M), i.key_pressed(egui::Key::O), i.key_pressed(egui::Key::P), i.key_pressed(egui::Key::N), i.key_pressed(egui::Key::G), i.key_pressed(egui::Key::R), i.key_pressed(egui::Key::Y), i.key_pressed(egui::Key::L), i.key_pressed(egui::Key::J), i.key_pressed(egui::Key::K),
            i.key_down(egui::Key::ArrowUp) || i.key_down(egui::Key::W), i.key_down(egui::Key::ArrowDown) || i.key_down(egui::Key::S), i.key_down(egui::Key::ArrowLeft) || i.key_down(egui::Key::A), i.key_down(egui::Key::ArrowRight) || i.key_down(egui::Key::D),
            i.modifiers.shift, i.modifiers.ctrl,
            [i.key_pressed(egui::Key::Num0), i.key_pressed(egui::Key::Num1), i.key_pressed(egui::Key::Num2), i.key_pressed(egui::Key::Num3), i.key_pressed(egui::Key::Num4), i.key_pressed(egui::Key::Num5), i.key_pressed(egui::Key::Num6), i.key_pressed(egui::Key::Num7), i.key_pressed(egui::Key::Num8), i.key_pressed(egui::Key::Num9)]
        ));

        for (idx, &pressed) in keys_0_9.iter().enumerate() { if pressed && self.focus == KeyboardFocus::Main { self.zoom_factor = match idx { 1 => 1.0, 0 => 0.1, 2 => 2.0, 3 => 4.0, 4 => 8.0, 5 => 12.0, 6 => 16.0, 7 => 20.0, 8 => 24.0, 9 => 32.0, _ => self.zoom_factor }; self.fit_to_screen = false; } }

        if esc {
            self.focus = match self.focus {
                KeyboardFocus::Editing(_) => if self.active_tab == Tab::Adjust { KeyboardFocus::Adjust } else { KeyboardFocus::Dither },
                KeyboardFocus::Light | KeyboardFocus::Color => KeyboardFocus::Adjust,
                KeyboardFocus::ModeSelection | KeyboardFocus::PosterizeMenu | KeyboardFocus::BayerSizeMenu | KeyboardFocus::GradientMapMenu => KeyboardFocus::Dither,
                KeyboardFocus::GradientPointEdit => KeyboardFocus::GradientMapMenu,
                _ => KeyboardFocus::Main,
            };
            self.show_export_window = false;
        }

        match self.focus {
            KeyboardFocus::Main => { if k_a { self.active_tab = Tab::Adjust; self.focus = KeyboardFocus::Adjust; } if k_d { self.active_tab = Tab::Dither; self.focus = KeyboardFocus::Dither; } }
            KeyboardFocus::Adjust => { if k_q { self.focus = KeyboardFocus::Light; } if k_e { self.focus = KeyboardFocus::Color; } if k_d { self.active_tab = Tab::Dither; self.focus = KeyboardFocus::Dither; } }
            KeyboardFocus::Light => {
                if k_e { self.focus = KeyboardFocus::Editing("exposure"); } if k_c { self.focus = KeyboardFocus::Editing("contrast"); } if k_h { self.focus = KeyboardFocus::Editing("highlights"); }
                if k_s { self.focus = KeyboardFocus::Editing("shadows"); } if k_b { self.focus = KeyboardFocus::Editing("blacks"); } if k_w { self.focus = KeyboardFocus::Editing("whites"); } if k_f { self.focus = KeyboardFocus::Editing("sharpness"); }
            }
            KeyboardFocus::Color => {
                if k_t { self.focus = KeyboardFocus::Editing("temperature"); } if k_e { self.focus = KeyboardFocus::Editing("tint"); } if k_s { self.focus = KeyboardFocus::Editing("saturation"); }
                if k_v { self.focus = KeyboardFocus::Editing("vibrance"); } if k_f { self.focus = KeyboardFocus::Editing("sharpness"); }
            }
            KeyboardFocus::Dither => {
                if k_m { self.focus = KeyboardFocus::ModeSelection; } if k_s { self.focus = KeyboardFocus::Editing("scale"); } if k_p { self.focus = KeyboardFocus::PosterizeMenu; }
                if k_t && self.settings.dither_type == 1.0 { self.focus = KeyboardFocus::Editing("threshold"); } if k_f && self.settings.dither_type == 3.0 { self.focus = KeyboardFocus::BayerSizeMenu; }
                if k_c && self.settings.dither_type != 1.0 { self.settings.dither_color = if self.settings.dither_color > 0.5 { 0.0 } else { 1.0 }; changed = true; } 
                if k_g { self.focus = KeyboardFocus::GradientMapMenu; } if k_a { self.active_tab = Tab::Adjust; self.focus = KeyboardFocus::Adjust; }
            }
            KeyboardFocus::ModeSelection => {
                let mut m = None; if k_a { m = Some(0.0); } if k_s { m = Some(1.0); } if k_d { m = Some(2.0); } if k_f { m = Some(3.0); } if k_g { m = Some(4.0); } if k_h { m = Some(5.0); } if k_j { m = Some(6.0); } if k_k { m = Some(7.0); } if k_l { m = Some(8.0); } if k_c { m = Some(9.0); }
                if let Some(val) = m { self.settings.dither_type = val; self.settings.dither_enabled = if val > 0.0 { 1.0 } else { 0.0 }; self.focus = KeyboardFocus::Dither; changed = true; }
            }
            KeyboardFocus::PosterizeMenu => { if k_e { self.settings.posterize_levels = if self.settings.posterize_levels > 0.0 { 0.0 } else { 4.0 }; changed = true; } if self.settings.posterize_levels > 0.0 { self.focus = KeyboardFocus::Editing("posterize"); } }
            KeyboardFocus::BayerSizeMenu => { let mut sz = None; if keys_0_9[2] { sz = Some(2.0); } if keys_0_9[3] { sz = Some(3.0); } if keys_0_9[4] { sz = Some(4.0); } if keys_0_9[8] { sz = Some(8.0); } if let Some(s) = sz { self.settings.bayer_size = s; self.focus = KeyboardFocus::Dither; changed = true; } }
            KeyboardFocus::GradientMapMenu => {
                if k_e { self.settings.grad_enabled = if self.settings.grad_enabled > 0.5 { 0.0 } else { 1.0 }; changed = true; }
                let now = ctx.input(|i| i.time);
                if now - self.last_edit_time > 0.166 {
                    if k_left || k_a {
                        if let Some(id) = self.selected_stop_id {
                            if let Some(idx) = self.gradient_stops.iter().position(|s| s.id == id) {
                                if idx > 0 { self.selected_stop_id = Some(self.gradient_stops[idx-1].id); self.last_edit_time = now; }
                            }
                        }
                    }
                    if k_right || k_d {
                        if let Some(id) = self.selected_stop_id {
                            if let Some(idx) = self.gradient_stops.iter().position(|s| s.id == id) {
                                if idx < self.gradient_stops.len() - 1 { self.selected_stop_id = Some(self.gradient_stops[idx+1].id); self.last_edit_time = now; }
                            }
                        }
                    }
                }
                if space { self.focus = KeyboardFocus::GradientPointEdit; }
                if k_n { let nid = self.next_stop_id; self.next_stop_id += 1; self.gradient_stops.push(GradientStop { id: nid, pos: 0.5, color: egui::Color32::GRAY }); self.selected_stop_id = Some(nid); self.gradient_stops.sort_by(|a,b| a.pos.partial_cmp(&b.pos).unwrap()); Self::generate_gradient_data(&self.gradient_stops, &mut self.gradient_data); if let Some(q) = &self.queue { self.pipeline.update_gradient(q, &self.gradient_data); } changed = true; }
                if k_b { if let Some(id) = self.selected_stop_id { if self.gradient_stops.len() > 2 { self.gradient_stops.retain(|s| s.id != id); self.selected_stop_id = self.gradient_stops.first().map(|s| s.id); Self::generate_gradient_data(&self.gradient_stops, &mut self.gradient_data); if let Some(q) = &self.queue { self.pipeline.update_gradient(q, &self.gradient_data); } changed = true; } } }
            }
            KeyboardFocus::GradientPointEdit => {
                if space { self.focus = KeyboardFocus::GradientMapMenu; }
                let now = ctx.input(|i| i.time);
                if now - self.last_edit_time > 0.166 {
                    let mut st_ch = false;
                    if let Some(id) = self.selected_stop_id {
                        if let Some(stop) = self.gradient_stops.iter_mut().find(|s| s.id == id) {
                            let mut hsva = egui::ecolor::Hsva::from(stop.color);
                            let h_step = if shift { 1.0/360.0 } else { 10.0/360.0 };
                            let sv_step = if shift { 0.01 } else { 0.1 };
                            if k_r { hsva.h = (hsva.h + h_step).fract(); st_ch = true; }
                            if k_t { hsva.s = (hsva.s + sv_step).clamp(0.0, 1.0); st_ch = true; }
                            if k_y { hsva.v = (hsva.v + sv_step).clamp(0.0, 1.0); st_ch = true; }
                            if k_f { hsva.h = (hsva.h - h_step + 1.0).fract(); st_ch = true; }
                            if k_g { hsva.s = (hsva.s - sv_step).clamp(0.0, 1.0); st_ch = true; }
                            if k_h { hsva.v = (hsva.v - sv_step).clamp(0.0, 1.0); st_ch = true; }
                            if k_left || k_a { stop.pos = (stop.pos - 0.01).clamp(0.0, 1.0); st_ch = true; }
                            if k_right || k_d { stop.pos = (stop.pos + 0.01).clamp(0.0, 1.0); st_ch = true; }
                            if st_ch { stop.color = egui::Color32::from(hsva); }
                        }
                    }
                    if st_ch {
                        self.gradient_stops.sort_by(|a,b| a.pos.partial_cmp(&b.pos).unwrap());
                        Self::generate_gradient_data(&self.gradient_stops, &mut self.gradient_data);
                        if let Some(queue) = &self.queue { self.pipeline.update_gradient(queue, &self.gradient_data); }
                        changed = true; self.last_edit_time = now;
                    }
                }
            }
            KeyboardFocus::Editing(id) => {
                if space { self.focus = if self.active_tab == Tab::Adjust { KeyboardFocus::Adjust } else { KeyboardFocus::Dither }; }
                let delta = if k_right || k_up { 1.0 } else if k_left || k_down { -1.0 } else { 0.0 };
                if delta != 0.0 {
                    let now = ctx.input(|i| i.time);
                    if now - self.last_edit_time > 0.166 {
                        let mut act_step = if shift { 0.1 } else { 0.05 };
                        if id == "exposure" { act_step = if shift { 0.5 } else { 0.15 }; }
                        match id {
                            "exposure" => self.settings.exposure = (self.settings.exposure + delta * act_step).clamp(-5.0, 5.0),
                            "contrast" => self.settings.contrast = (self.settings.contrast + delta * act_step).clamp(0.0, 2.0),
                            "highlights" => self.settings.highlights = (self.settings.highlights + delta * act_step).clamp(-1.0, 1.0),
                            "shadows" => self.settings.shadows = (self.settings.shadows + delta * act_step).clamp(-1.0, 1.0),
                            "whites" => self.settings.whites = (self.settings.whites + delta * act_step).clamp(-1.0, 1.0),
                            "blacks" => self.settings.blacks = (self.settings.blacks + delta * act_step).clamp(-1.0, 1.0),
                            "sharpness" => self.settings.sharpness = (self.settings.sharpness + delta * act_step).clamp(0.0, 2.0),
                            "temperature" => self.settings.temperature = (self.settings.temperature + delta * act_step).clamp(-1.0, 1.0),
                            "tint" => self.settings.tint = (self.settings.tint + delta * act_step).clamp(-1.0, 1.0),
                            "saturation" => self.settings.saturation = (self.settings.saturation + delta * act_step).clamp(0.0, 2.0),
                            "vibrance" => self.settings.vibrance = (self.settings.vibrance + delta * act_step).clamp(-1.0, 1.0),
                            "scale" => self.settings.dither_scale = (self.settings.dither_scale + delta * act_step * 10.0).clamp(1.0, 32.0),
                            "threshold" => self.settings.dither_threshold = (self.settings.dither_threshold + delta * act_step).clamp(0.0, 1.0),
                            "posterize" => self.settings.posterize_levels = (self.settings.posterize_levels + delta * act_step * 10.0).clamp(0.0, 64.0),
                            _ => {}
                        }
                        self.last_edit_time = now; changed = true;
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top_shortcuts").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("[{}]", match self.focus {
                    KeyboardFocus::Main => "MAIN", KeyboardFocus::Adjust => "ADJUST", KeyboardFocus::Light => "ADJUST > LIGHT", KeyboardFocus::Color => "ADJUST > COLOR", KeyboardFocus::Dither => "DITHER", KeyboardFocus::ModeSelection => "SELECT MODE", KeyboardFocus::PosterizeMenu => "POSTERIZE", KeyboardFocus::BayerSizeMenu => "BAYER SIZE", KeyboardFocus::GradientMapMenu => "GRADIENT MAP", KeyboardFocus::GradientPointEdit => "EDIT POINT", KeyboardFocus::Editing(_) => "EDITING",
                }));
                ui.separator();
                let shortcuts = match self.focus {
                    KeyboardFocus::Main => "A:Adjust  D:Dither  Esc:Back  0-9:Zoom",
                    KeyboardFocus::Adjust => "Q:Light  E:Color  Esc:Back",
                    KeyboardFocus::Light => "E:Exp C:Cont H:High S:Shad B:Black W:White F:Sharp Esc:Back",
                    KeyboardFocus::Color => "T:Temp E:Tint S:Sat V:Vib F:Sharp Esc:Back",
                    KeyboardFocus::Dither => "M:Mode S:Scale P:Post T:Thresh F:Bayer C:Color G:Ramp Esc:Back",
                    KeyboardFocus::ModeSelection => "A:None S:Thresh D:Rand F:Bayer G:Blue H:Diff J:Stucki K:Atkin L:Grad C:Latt",
                    KeyboardFocus::PosterizeMenu => "E:Toggle  Esc:Back",
                    KeyboardFocus::BayerSizeMenu => "2,3,4,8:Size  Esc:Back",
                    KeyboardFocus::GradientMapMenu => "E:Toggle  A/D:Navigate  Space:Edit  N:Add  B:Remove  Esc:Back",
                    KeyboardFocus::GradientPointEdit => "R,T,Y/F,G,H:HSB +/-  A/D:Move  Shift:Fine  Space:Done",
                    KeyboardFocus::Editing(_) => "ARROWS/WASD:Change  Shift:Fast  Space:OK",
                };
                ui.label(shortcuts);
            });
        });

        if let KeyboardFocus::Editing(id) = self.focus {
            egui::Area::new(egui::Id::new("edit_overlay")).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)).show(ctx, |ui| {
                let frame = egui::Frame::none().fill(egui::Color32::BLACK).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 65))).inner_margin(20.0);
                frame.show(ui, |ui| {
                    let val = match id {
                        "exposure" => self.settings.exposure, "contrast" => self.settings.contrast, "highlights" => self.settings.highlights, "shadows" => self.settings.shadows, "whites" => self.settings.whites, "blacks" => self.settings.blacks, "sharpness" => self.settings.sharpness, "temperature" => self.settings.temperature, "tint" => self.settings.tint, "saturation" => self.settings.saturation, "vibrance" => self.settings.vibrance, "scale" => self.settings.dither_scale, "threshold" => self.settings.dither_threshold, "posterize" => self.settings.posterize_levels, _ => 0.0,
                    };
                    ui.heading(format!("{}: {:.2}", id.to_uppercase(), val));
                });
            });
        }

        ctx.input(|i| { if !i.raw.dropped_files.is_empty() { if let Some(path) = i.raw.dropped_files[0].path.as_ref() { if let Ok(img) = image_io::load_from_path(path) { self.load_image_to_gpu(ctx, img); } } } });

        egui::SidePanel::left("control_panel").resizable(true).default_width(300.0).show(ctx, |ui| {
            ui.heading("VibeDither v0.3"); ui.separator();
            ui.vertical(|ui| {
                ui.label("IMAGE CONTROLS");
                if ui.button("Load Image").clicked() { if let Some(path) = rfd::FileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "webp"]).pick_file() { if let Ok(img) = image_io::load_from_path(&path) { self.load_image_to_gpu(ctx, img); } } }
                if ui.button("Paste from Clipboard").clicked() { if let Some(img) = image_io::get_clipboard_image() { self.load_image_to_gpu(ctx, img); } }
                if ui.button("Export Image").clicked() { self.show_export_window = true; }
                ui.separator();
                ui.horizontal(|ui| { ui.selectable_value(&mut self.active_tab, Tab::Adjust, "ADJUST"); ui.selectable_value(&mut self.active_tab, Tab::Dither, "DITHER"); });
                ui.separator();
                let mut side_changed = false;
                match self.active_tab {
                    Tab::Adjust => {
                        ui.label("BASIC ADJUSTMENTS");
                        ui.group(|ui| {
                            ui.label("Light");
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.exposure, -5.0..=5.0).text("Exposure")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.contrast, 0.0..=2.0).text("Contrast")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.highlights, -1.0..=1.0).text("Highlights")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.shadows, -1.0..=1.0).text("Shadows")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.whites, -1.0..=1.0).text("Whites")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.blacks, -1.0..=1.0).text("Blacks")).changed();
                        });
                        ui.group(|ui| {
                            ui.label("Color");
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.temperature, -1.0..=1.0).text("Temperature")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.tint, -1.0..=1.0).text("Tint")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.vibrance, -1.0..=1.0).text("Vibrance")).changed();
                            side_changed |= ui.add(egui::Slider::new(&mut self.settings.saturation, 0.0..=2.0).text("Saturation")).changed();
                        });
                        ui.group(|ui| { ui.label("Detail"); side_changed |= ui.add(egui::Slider::new(&mut self.settings.sharpness, 0.0..=2.0).text("Sharpness")).changed(); });
                        if ui.button("Reset Adjustments").clicked() { self.settings = ColorSettings::default(); side_changed = true; }
                        ui.separator(); ui.label("RGB CURVES");
                        let mut curves_changed = false;
                        ui.vertical(|ui| {
                            let size = egui::vec2(ui.available_width(), 150.0);
                            let (rect, response) = ui.allocate_at_least(size, egui::Sense::click_and_drag());
                            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(15, 15, 15));
                            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40)));
                            for i in 1..4 { let x = rect.left() + rect.width() * (i as f32 / 4.0); let y = rect.top() + rect.height() * (i as f32 / 4.0); ui.painter().line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(0.5, egui::Color32::from_rgb(30, 30, 30))); ui.painter().line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(0.5, egui::Color32::from_rgb(30, 30, 30))); }
                            let lut = spline::interpolate_spline(&self.curve_points);
                            let mut points: Vec<egui::Pos2> = Vec::new();
                            for i in 0..256 { let val = lut[i]; let x = rect.left() + (i as f32 / 255.0) * rect.width(); let y = rect.bottom() - (val as f32 / 255.0) * rect.height(); points.push(egui::pos2(x, y)); }
                            ui.painter().add(egui::Shape::line(points, egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 65))));
                            if let Some(pos) = response.interact_pointer_pos() {
                                let x_norm = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                                let y_norm = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
                                if response.drag_started() {
                                    let mut closest_idx = None; let mut min_dist = 0.05;
                                    for (idx, p) in self.curve_points.iter().enumerate() { let dist = (p.x - x_norm).abs(); if dist < min_dist { min_dist = dist; closest_idx = Some(idx); } }
                                    self.dragging_point_idx = closest_idx;
                                }
                                if response.dragged() {
                                    if let Some(idx) = self.dragging_point_idx {
                                        if idx == 0 || idx == self.curve_points.len() - 1 { self.curve_points[idx].y = y_norm; }
                                        else { let min_x = self.curve_points[idx-1].x + 0.001; let max_x = self.curve_points[idx+1].x - 0.001; self.curve_points[idx] = egui::pos2(x_norm.clamp(min_x, max_x), y_norm); }
                                        curves_changed = true;
                                    }
                                }
                                if response.drag_stopped() { self.dragging_point_idx = None; }
                                if response.clicked() && !response.dragged() {
                                    let mut exists = false; for p in &self.curve_points { if (p.x - x_norm).abs() < 0.02 { exists = true; break; } }
                                    if !exists { self.curve_points.push(egui::pos2(x_norm, y_norm)); self.curve_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap()); curves_changed = true; }
                                }
                            }
                            if response.secondary_clicked() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let x_norm = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                                    let mut to_remove = None;
                                    for (idx, p) in self.curve_points.iter().enumerate() { if (p.x - x_norm).abs() < 0.03 && idx != 0 && idx != self.curve_points.len() - 1 { to_remove = Some(idx); break; } }
                                    if let Some(idx) = to_remove { self.curve_points.remove(idx); curves_changed = true; }
                                }
                            }
                            for p in &self.curve_points { let px = rect.left() + p.x * rect.width(); let py = rect.bottom() - p.y * rect.height(); ui.painter().circle_filled(egui::pos2(px, py), 3.0, egui::Color32::WHITE); }
                            if ui.button("Reset Curves").clicked() { self.curve_points = vec![egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)]; curves_changed = true; }
                        });
                        if curves_changed {
                            let lut = spline::interpolate_spline(&self.curve_points);
                            for i in 0..256 { self.curves_data[i * 4] = lut[i]; self.curves_data[i * 4 + 1] = lut[i]; self.curves_data[i * 4 + 2] = lut[i]; }
                            if let Some(queue) = &self.queue { self.pipeline.update_curves(queue, &self.curves_data); }
                            side_changed = true;
                        }
                    },
                    Tab::Dither => {
                        ui.label("DITHERING CONTROLS");
                        let d_type = self.settings.dither_type as usize;
                        let d_names = ["None", "Threshold", "Random", "Bayer", "Blue Noise", "Diffusion Approx", "Stucki", "Atkinson", "Gradient Based", "Lattice-Boltzmann"];
                        egui::ComboBox::from_label("Algorithm").selected_text(d_names[d_type.min(d_names.len() - 1)]).show_ui(ui, |ui| {
                            for (i, name) in d_names.iter().enumerate() { if ui.selectable_label(d_type == i, *name).clicked() { self.settings.dither_type = i as f32; self.settings.dither_enabled = if i > 0 { 1.0 } else { 0.0 }; self.settings.dither_color = 0.0; side_changed = true; } }
                        });
                        ui.add_enabled_ui(d_type > 0, |ui| {
                            let mut scale_int = self.settings.dither_scale as i32; if ui.add(egui::Slider::new(&mut scale_int, 1..=32).text("Pixel Scale")).changed() { self.settings.dither_scale = scale_int as f32; side_changed = true; }
                            if d_type == 1 { side_changed |= ui.add(egui::Slider::new(&mut self.settings.dither_threshold, 0.0..=1.0).text("Threshold")).changed(); }
                            ui.group(|ui| {
                                ui.label("Posterize");
                                let mut use_p = self.settings.posterize_levels > 0.0;
                                if ui.checkbox(&mut use_p, "Enabled").changed() { self.settings.posterize_levels = if use_p { 4.0 } else { 0.0 }; side_changed = true; }
                                ui.add_enabled_ui(use_p, |ui| { side_changed |= ui.add(egui::Slider::new(&mut self.settings.posterize_levels, 2.0..=64.0).text("Levels")).changed(); });
                            });
                            if d_type == 3 { ui.horizontal(|ui| { ui.label("Matrix Size:"); let sizes = [2, 3, 4, 8]; for s in sizes { if ui.selectable_label(self.settings.bayer_size as i32 == s, format!("{}x{}", s, s)).clicked() { self.settings.bayer_size = s as f32; side_changed = true; } } }); }
                            if d_type >= 1 { let mut color_d = self.settings.dither_color > 0.5; if ui.checkbox(&mut color_d, "Color Dithering").changed() { self.settings.dither_color = if color_d { 1.0 } else { 0.0 }; side_changed = true; } }
                        });
                        ui.separator(); ui.label("GRADIENT REMAP");
                        let mut grad_e = self.settings.grad_enabled > 0.5;
                        if ui.checkbox(&mut grad_e, "Enable Gradient Remap").changed() { self.settings.grad_enabled = if grad_e { 1.0 } else { 0.0 }; side_changed = true; }
                        ui.add_enabled_ui(grad_e, |ui| {
                            let mut stops_ch = false;
                            ui.vertical(|ui| {
                                let ramp_h = 20.0; let (ramp_r, _) = ui.allocate_at_least(egui::vec2(ui.available_width(), ramp_h), egui::Sense::hover()); let _rect = ui.allocate_space(egui::vec2(ui.available_width(), 15.0));
                                for i in 0..255 {
                                    let t0 = i as f32 / 255.0; let t1 = (i + 1) as f32 / 255.0; let x0 = ramp_r.left() + t0 * ramp_r.width(); let x1 = ramp_r.left() + t1 * ramp_r.width();
                                    let c0 = egui::Color32::from_rgba_unmultiplied(self.gradient_data[i * 4], self.gradient_data[i * 4 + 1], self.gradient_data[i * 4 + 2], 255);
                                    ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(x0, ramp_r.top()), egui::pos2(x1, ramp_r.bottom())), 0.0, c0);
                                }
                                ui.painter().rect_stroke(ramp_r, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)));
                                let mut active_id = self.selected_stop_id; let mut dragged_id = None; let mut new_p_val = 0.0;
                                for stop in self.gradient_stops.iter() {
                                    let x = ramp_r.left() + stop.pos * ramp_r.width(); let y = ramp_r.bottom() + 2.0; let is_sel = Some(stop.id) == active_id; let color = if is_sel { egui::Color32::WHITE } else { egui::Color32::from_rgb(150, 150, 150) };
                                    ui.painter().add(egui::Shape::convex_polygon(vec![egui::pos2(x, y), egui::pos2(x - 5.0, y + 8.0), egui::pos2(x + 5.0, y + 8.0)], color, egui::Stroke::NONE));
                                    let h_res = ui.interact(egui::Rect::from_center_size(egui::pos2(x, y + 4.0), egui::vec2(10.0, 10.0)), egui::Id::new(("grad", stop.id)), egui::Sense::click_and_drag());
                                    if h_res.clicked() { active_id = Some(stop.id); }
                                    if h_res.dragged() { active_id = Some(stop.id); new_p_val = (stop.pos + h_res.drag_delta().x / ramp_r.width()).clamp(0.0, 1.0); dragged_id = Some(stop.id); }
                                }
                                if let Some(id) = dragged_id { if let Some(stop) = self.gradient_stops.iter_mut().find(|s| s.id == id) { stop.pos = new_p_val; stops_ch = true; } }
                                self.selected_stop_id = active_id;
                                ui.horizontal(|ui| {
                                    if ui.button("+").clicked() { let nid = self.next_stop_id; self.next_stop_id += 1; self.gradient_stops.push(GradientStop { id: nid, pos: 0.5, color: egui::Color32::GRAY }); self.selected_stop_id = Some(nid); stops_ch = true; }
                                    if let Some(id) = self.selected_stop_id {
                                        if self.gradient_stops.len() > 2 { if ui.button("-").clicked() { self.gradient_stops.retain(|s| s.id != id); self.selected_stop_id = self.gradient_stops.first().map(|s| s.id); stops_ch = true; } }
                                        ui.separator();
                                        if let Some(stop) = self.gradient_stops.iter_mut().find(|s| s.id == id) { if ui.color_edit_button_srgba(&mut stop.color).changed() { stops_ch = true; } ui.add(egui::DragValue::new(&mut stop.pos).speed(0.01).clamp_range(0.0..=1.0)); }
                                    }
                                });
                            });
                            if stops_ch { self.gradient_stops.sort_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap()); Self::generate_gradient_data(&self.gradient_stops, &mut self.gradient_data); if let Some(q) = &self.queue { self.pipeline.update_gradient(q, &self.gradient_data); } side_changed = true; }
                        });
                    },
                }
                if side_changed || changed {
                    if let (Some(device), Some(queue), Some(input), Some(output)) = (&self.device, &self.queue, &self.input_texture, &self.output_texture) {
                        self.pipeline.render(device, queue, &input.create_view(&wgpu::TextureViewDescriptor::default()), &output.create_view(&wgpu::TextureViewDescriptor::default()), &self.settings);
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("zoom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Zoom: {:.0}%", self.zoom_factor * 100.0));
                egui::ComboBox::from_id_source("zoom_selector").selected_text(format!("{:.0}%", self.zoom_factor * 100.0)).show_ui(ui, |ui| {
                    let zooms = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0];
                    for z in zooms { if ui.selectable_value(&mut self.zoom_factor, z, format!("{:.0}%", z * 100.0)).clicked() { self.fit_to_screen = false; } }
                });
                if ui.button("100%").clicked() { self.zoom_factor = 1.0; self.fit_to_screen = false; }
                if ui.button("Fit to Screen").clicked() { self.fit_to_screen = true; }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex_id) = self.egui_texture_id {
                let img_size = self.current_image.as_ref().map(|img| egui::vec2(img.width() as f32, img.height() as f32)).unwrap_or(egui::Vec2::ZERO);
                let (rect, resp) = ui.allocate_at_least(ui.available_size(), egui::Sense::click_and_drag());
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    let old_z = self.zoom_factor; self.zoom_factor = (self.zoom_factor * (1.0 + scroll * 0.002)).clamp(0.1, 32.0); self.fit_to_screen = false;
                    if let Some(m_pos) = ui.input(|i| i.pointer.hover_pos()) { let rel = m_pos - rect.center() - self.pan_offset; self.pan_offset -= rel * (self.zoom_factor / old_z - 1.0); }
                }
                if resp.dragged_by(egui::PointerButton::Primary) { self.pan_offset += resp.drag_delta(); }
                let d_size = if self.fit_to_screen { let r = (rect.width() / img_size.x).min(rect.height() / img_size.y); img_size * r } else { img_size * self.zoom_factor };
                ui.set_clip_rect(rect); ui.painter().image(tex_id, egui::Rect::from_center_size(rect.center() + self.pan_offset, d_size), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
            } else { ui.centered_and_justified(|ui| { ui.label("Drag and drop an image or use 'Load Image'"); }); }
        });

        if self.show_export_window {
            let mut close = false;
            egui::Window::new("Export Settings").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("FORMAT");
                    ui.horizontal(|ui| { ui.selectable_value(&mut self.export_settings.format, ExportFormat::Png, "PNG"); ui.selectable_value(&mut self.export_settings.format, ExportFormat::Jpg, "JPG"); ui.selectable_value(&mut self.export_settings.format, ExportFormat::Webp, "WEBP"); });
                    ui.separator(); ui.label("SETTINGS");
                    if self.export_settings.format == ExportFormat::Jpg || self.export_settings.format == ExportFormat::Webp { ui.add(egui::Slider::new(&mut self.export_settings.compression, 0.0..=1.0).text("Quality")); } else { ui.add(egui::Slider::new(&mut self.export_settings.compression, 0.0..=1.0).text("Compression (File Size)")); }
                    ui.add_enabled(self.export_settings.format != ExportFormat::Jpg, egui::Checkbox::new(&mut self.export_settings.transparency, "Enable Transparency"));
                    ui.separator(); ui.horizontal(|ui| { ui.label("SIZE"); if ui.selectable_label(self.export_settings.use_percentage, "%").clicked() { self.export_settings.use_percentage = true; } if ui.selectable_label(!self.export_settings.use_percentage, "px").clicked() { self.export_settings.use_percentage = false; } });
                    if self.export_settings.use_percentage { if ui.add(egui::Slider::new(&mut self.export_settings.percentage, 0.1..=5.0).text("Scale")).changed() { if let Some(img) = &self.current_image { self.export_settings.width_px = (img.width() as f32 * self.export_settings.percentage) as u32; self.export_settings.height_px = (img.height() as f32 * self.export_settings.percentage) as u32; } } }
                    else {
                        ui.horizontal(|ui| {
                            let mut w = self.export_settings.width_px; let mut h = self.export_settings.height_px;
                            if ui.add(egui::DragValue::new(&mut w).clamp_range(1..=16384).prefix("W: ")).changed() { if self.export_settings.link_aspect { if let Some(img) = &self.current_image { h = (w as f32 * (img.height() as f32 / img.width() as f32)) as u32; } } self.export_settings.width_px = w; self.export_settings.height_px = h; }
                            if ui.button(if self.export_settings.link_aspect { "🔗" } else { "🔓" }).clicked() { self.export_settings.link_aspect = !self.export_settings.link_aspect; }
                            if ui.add(egui::DragValue::new(&mut h).clamp_range(1..=16384).prefix("H: ")).changed() { if self.export_settings.link_aspect { if let Some(img) = &self.current_image { w = (h as f32 * (img.width() as f32 / img.height() as f32)) as u32; } } self.export_settings.width_px = w; self.export_settings.height_px = h; }
                        });
                    }
                    ui.separator();
                    ui.horizontal(|ui| { if ui.button("Cancel").clicked() { close = true; } if ui.button("Export").clicked() { self.export_image(); close = true; } });
                });
            });
            if close { self.show_export_window = false; }
        }
    }
}

fn color_add(c: egui::Color32, r: i16, g: i16, b: i16) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied((c.r() as i16 + r).clamp(0, 255) as u8, (c.g() as i16 + g).clamp(0, 255) as u8, (c.b() as i16 + b).clamp(0, 255) as u8, 255)
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
    for text_style in [egui::TextStyle::Body, egui::TextStyle::Monospace, egui::TextStyle::Button, egui::TextStyle::Heading, egui::TextStyle::Small] {
        style.text_styles.insert(text_style, egui::FontId::monospace(14.0));
    }
    ctx.set_style(style);
}
