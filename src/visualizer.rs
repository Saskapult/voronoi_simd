use std::{time::{Duration, Instant}, u8};
use eframe::egui;

pub fn run_visualizer() {
	let native_options = eframe::NativeOptions::default();
	eframe::run_native("Noise Visualizer", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))).unwrap();
}

#[derive(Default, Clone, Copy)]
struct VoronoiSettings {
	seed: u32,
	size_x: u32,
	size_y: u32,

	frequency: f32,
	offset_x: f32,
	offset_y: f32,
}

#[derive(Default)]
struct MyEguiApp {
	size_lock: bool,
	size_lock_scale: f32,
	last_display_size: [u32; 2],
	settings: VoronoiSettings,

	noise_texture: Option<egui::TextureHandle>,
	noise_state: GenerationState,
	last_time: Option<Duration>,
}

impl MyEguiApp {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
		// Restore app state using cc.storage (requires the "persistence" feature).
		// cc.storage.unwrap()
		// Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
		// for e.g. egui::PaintCallback.
		Self {
			settings: VoronoiSettings {
				frequency: 0.1,
				size_x: 128,
				size_y: 128,
				..Default::default()
			},
			last_display_size: [128, 128],
			size_lock: true,
			size_lock_scale: 0.5,
			..Default::default()
		}
		// Self::default()
	}

	fn generate_noise(&mut self) {
		let (sender, receiver) = std::sync::mpsc::channel();

		let settings = self.settings.clone();
		std::thread::spawn(move || {
			let mut data = Vec::with_capacity((settings.size_x * settings.size_y) as usize);
			for y in 0..settings.size_y {
				for x in 0..settings.size_x {
					data.push(crate::voronoi::voronoi_basic(
						settings.seed, 
						settings.frequency, 
						settings.offset_x + x as f32, 
						settings.offset_y + y as f32,
					));
				}
			}
			match sender.send((data, settings)) {
				Ok(_) => {},
				Err(e) => println!("Error sending noise values: {e}"),
			}
		});

		self.noise_state = GenerationState::Working(Instant::now(), receiver);
	}
}

impl eframe::App for MyEguiApp {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		egui_extras::install_image_loaders(ctx);

		// Check if noise done
		if let GenerationState::Working(t, h) = &self.noise_state {
			if let Ok((data, settings)) = h.try_recv() {
				self.last_time = Some(t.elapsed());
				// egui::Image::from_bytes(uri, bytes)
				let pixels = data.into_iter()
					.map(|v| (v * u8::MAX as f32) as u8)
					.map(|v| egui::Color32::from_rgb(v, v, v))
					.collect::<Vec<_>>();
				let image = egui::ColorImage {
					size: [settings.size_x as usize, settings.size_y as usize],
					pixels,
				};
				self.noise_texture = Some(ctx.load_texture("noise texture", image, egui::TextureOptions::LINEAR));

				self.noise_state = GenerationState::Finished;
			}
		}

		egui::SidePanel::left("Noise Settings").show(ctx, |ui| {
			ui.add(egui::Slider::new(&mut self.settings.offset_x, 0.0..=100.0).prefix("offset x: "));
			ui.add(egui::Slider::new(&mut self.settings.offset_y, 0.0..=100.0).prefix("offset y: "));

			ui.add(egui::Slider::new(&mut self.settings.frequency, 0.00001..=1.0).prefix("frequnecy: "));

			ui.add_enabled_ui(!self.size_lock, |ui| {
				ui.add(egui::Slider::new(&mut self.settings.size_x, 0..=self.last_display_size[0]).prefix("size x: "));
				ui.add(egui::Slider::new(&mut self.settings.size_y, 0..=self.last_display_size[1]).prefix("size y: "));
			});
			
			ui.checkbox(&mut self.size_lock, "Match size with window");
			ui.add_enabled_ui(self.size_lock, |ui| {
				ui.add(egui::Slider::new(&mut self.size_lock_scale, 0.0..=1.0).prefix("scale: "));
			});
			if self.size_lock {
				self.settings.size_x = (self.last_display_size[0] as f32 * self.size_lock_scale) as u32;
				self.settings.size_y = (self.last_display_size[1] as f32 * self.size_lock_scale) as u32;
			}

			ui.add_enabled_ui(!self.noise_state.is_working(), |ui| {
				ui.horizontal(|ui| {
					if ui.button("Generate").clicked() {
						self.generate_noise();
					}
					if !ui.is_enabled() {
						ui.spinner();
					}
				})
			});
			if let Some(d) = self.last_time {
				ui.label(format!("Generated in {:.4}ms", d.as_secs_f32() * 1000.0));
			}
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			let available_size = ui.available_size();
			self.last_display_size[0] = available_size.x as u32;
			self.last_display_size[1] = available_size.y as u32;

			if let Some(t) = &self.noise_texture {
				let (rect, _) = ui.allocate_at_least(available_size, egui::Sense::click());
				let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0));
				ui.painter().image(t.id(), rect, uv, egui::Color32::WHITE);
				// ui.image(t);
			} else {
				ui.heading("Let's make some noise!");				
			}
		});
	}

	// fn save(&mut self, storage: &mut dyn eframe::Storage) {
	// 	storage.set_string("seed", self.settings.seed);

	// }
}

#[derive(Default)]
enum GenerationState {
	#[default]
	Finished,
	Working(Instant, std::sync::mpsc::Receiver<(Vec<f32>, VoronoiSettings)>),
}
impl GenerationState {
	pub fn is_working(&self) -> bool {
		if let Self::Finished = self {
			false
		} else {
			true
		}
	}
}
