// Currently broken!!
// puffin_egui has to be updated
//
// This example needs the puffin feature enabled to work
// you can test this with:
// cargo run --example puffin --features puffin

use eframe::NativeOptions;
use puffin_egui::puffin;

fn main() {
    puffin::set_scopes_on(true);
    // Initialize the logger
    egui_logger::builder()
        .init()
        .expect("Error initializing logger");

    let options = NativeOptions::default();

    eframe::run_native("egui_logger", options, Box::new(|_cc| Ok(Box::new(MyApp)))).unwrap();
}

#[derive(Default)]
struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        puffin::GlobalProfiler::lock().new_frame();
        puffin_egui::profiler_window(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(feature = "puffin")]
            puffin::profile_scope!("Render UI");
            if ui.button("This produces Debug Info").clicked() {
                log::debug!("Very verbose Debug Info")
            }
            if ui.button("This produces an Info").clicked() {
                log::info!("Some Info");
            }
            if ui.button("This produces an Error").clicked() {
                log::error!("Error doing Something");
            }
            if ui.button("This produces a Warning").clicked() {
                log::warn!("Warn about something")
            }
            if ui.button("This produces 1000 Warnings").clicked() {
                (1..1000).for_each(|x| log::warn!("Warn: {}", x));
                log::warn!("Warn about something")
            }
            if ui.button("This produces 100_000 Warnings").clicked() {
                (1..100_000).for_each(|x| log::warn!("Warn: {}", x));
                log::warn!("Warn about something")
            }
        });
        egui::Window::new("Log").show(ctx, |ui| {
            // draws the actual logger ui
            egui_logger::LoggerUi::default()
                .enable_regex(true) // enables regex, default is true
                .show(ui)
        });
    }
}
