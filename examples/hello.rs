use eframe::NativeOptions;

fn main() {
    // Initilize the logger
    egui_logger::init_with_max_level(log::LevelFilter::Debug).expect("Error initializing logger");

    let options = NativeOptions::default();

    eframe::run_native(
        "egui_logger",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    ).unwrap();
}

#[derive(Default)]
struct MyApp;

impl eframe::App for MyApp {
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
            // draws the actual logger ui
            egui_logger::logger_ui(ui);
        });
    }
}
