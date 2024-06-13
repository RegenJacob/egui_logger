use eframe::NativeOptions;

fn main() {
    // Create an EguiLogger; multi_log will take care of initialization.
    let egui_logger = Box::new(egui_logger::builder().build());

    // And add another one.
    let env_logger = Box::new(env_logger::builder().default_format().build());

    multi_log::MultiLogger::init(vec![egui_logger, env_logger], log::Level::Debug)
        .expect("Error initializing multi_logger");

    eframe::run_native(
        "egui_logger",
        NativeOptions::default(),
        Box::new(|_cc| Box::new(MultiLogApp)),
    )
    .expect("Couldn't run eframe app");
}

#[derive(Default)]
struct MultiLogApp;

impl eframe::App for MultiLogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
        });
        egui::Window::new("Log").show(ctx, |ui| {
            egui_logger::logger_ui(ui);
        });
    }
}
