use eframe::NativeOptions;

fn main() {
    // Initilize the logger
    egui_logger::init().expect("Error initializing logger");

    let options = NativeOptions::default();

    eframe::run_native(
        "egui_logger",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
    .unwrap();
}

#[derive(Default)]
struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("This produces tracing level logs").clicked() {
                log::trace!("Something insignificant probably happend");
            }
            if ui.button("This produces debug level logs").clicked() {
                log::debug!("Something that might be important to fix a bug");
            }
            if ui.button("This produces info level logs").clicked() {
                log::info!("Something worth mentioning has happend");
            }
            if ui.button("This produces warning level logs").clicked() {
                log::warn!("Warn about something")
            }
            if ui.button("This produces error level logs").clicked() {
                log::error!("Error doing something");
            }
        });
        egui::Window::new("Log").show(ctx, |ui| {
            // draws the actual logger ui
            egui_logger::logger_ui(ui);
        });
    }
}
