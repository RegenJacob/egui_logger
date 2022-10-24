use std::sync::Mutex;

use egui::Color32;
use log::{Level, SetLoggerError};

struct EguiLogger;

impl log::Log for EguiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
            let mut log = LOG.lock().unwrap();

            let mut l: Vec<String> = log.clone();
            l.push(format!("{}: {}", record.level(), record.args()));

            *log = l
        }
    }

    fn flush(&self) {}
}

/// Initilizes the global logger.
/// Should be called very early in the program
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&EguiLogger)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
}

trait Logger {
    fn write();
}

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

static LOGGER_UI: once_cell::sync::Lazy<Mutex<LoggerUi>> =
    once_cell::sync::Lazy::new(Default::default);

#[derive(Default)]
struct LoggerUi;

impl LoggerUi {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Log");
        egui::ScrollArea::vertical().show(ui, |ui| {
            let logs = LOG.lock().unwrap();
            logs.iter().for_each(|s| {
                if s.starts_with("WARN") {
                    ui.colored_label(Color32::YELLOW, s);
                } else if s.starts_with("ERROR") {
                    ui.colored_label(Color32::RED, s);
                } else {
                    ui.label(s);
                }
            })
                
        });
    }
}

/// Draws the logger ui 
/// has to be called after [`init()`](init());
pub fn logger_ui(ui: &mut egui::Ui) {
    let mut logger_ui = LOGGER_UI.lock().unwrap();

    logger_ui.ui(ui);
}
