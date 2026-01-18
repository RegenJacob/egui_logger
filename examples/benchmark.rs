use eframe::NativeOptions;
use std::collections::VecDeque;
use std::time::Instant;

fn main() {
    egui_logger::builder()
        .max_level(log::LevelFilter::Debug)
        .init()
        .expect("Error initializing logger");

    // Generate 1000 logs at startup (250 each of error/warn/info/debug)
    for i in 0..250 {
        log::error!("Error message {i:03}");
        log::warn!("Warning message {i:03}");
        log::info!("Info message {i:03}");
        log::debug!("Debug message {i:03}");
    }

    let options = NativeOptions::default();
    eframe::run_native(
        "egui_logger benchmark",
        options,
        Box::new(|_cc| Ok(Box::new(BenchmarkApp::default()))),
    )
    .unwrap();
}

struct BenchmarkApp {
    frame_times: VecDeque<f64>,
    log_counter: u32,
}

impl Default for BenchmarkApp {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(60),
            log_counter: 250,
        }
    }
}

impl eframe::App for BenchmarkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui_logger Benchmark");
            ui.label("1000 logs generated (250 each: error, warn, info, debug)");
            ui.separator();
            ui.label("Test scenarios (use UI controls in Log window):");
            ui.label("1. All displayed: default state");
            ui.label("2. Levels filtered: Log Levels menu");
            ui.label("3. Search filter: search box");
            ui.separator();

            let avg_ms = if self.frame_times.is_empty() {
                0.0
            } else {
                self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
            };
            ui.label(format!("Avg render time (last 60 frames): {avg_ms:.3} ms"));

            ui.separator();
            if ui.button("Add 100 logs").clicked() {
                for i in 0..25 {
                    let n = self.log_counter + i;
                    log::error!("Error message {n:03}");
                    log::warn!("Warning message {n:03}");
                    log::info!("Info message {n:03}");
                    log::debug!("Debug message {n:03}");
                }
                self.log_counter += 25;
            }
        });

        egui::Window::new("Log").show(ctx, |ui| {
            let start = Instant::now();
            egui_logger::logger_ui()
                .enable_cache_layouts(true)
                .max_log_length(2000)
                .show(ui);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;

            if self.frame_times.len() >= 60 {
                self.frame_times.pop_front();
            }
            self.frame_times.push_back(elapsed);

            let avg_ms = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
            println!("logger_ui render: {elapsed:.3} ms (avg: {avg_ms:.3} ms)");
        });
    }
}
