use std::sync::Mutex;

use egui::Color32;
use log::SetLoggerError;

struct EguiLogger;

impl log::Log for EguiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::STATIC_MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            //println!("{}: {}", record.level(), record.args());
            let mut log = LOG.lock().unwrap();

            let mut l: Vec<(log::Level, String)> = log.clone();
            l.push((record.level(), record.args().to_string()));

            *log = l
        }
    }

    fn flush(&self) {}
}

/// Initilizes the global logger.
/// Should be called very early in the program
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&EguiLogger).map(|()| log::set_max_level(log::LevelFilter::Info))
}

static LOG: Mutex<Vec<(log::Level, String)>> = Mutex::new(Vec::new());

static LOGGER_UI: once_cell::sync::Lazy<Mutex<LoggerUi>> =
    once_cell::sync::Lazy::new(Default::default);

struct LoggerUi {
    loglevel: log::Level,
    search_term: String,
}

impl Default for LoggerUi {
    fn default() -> Self {
        Self {
            loglevel: log::Level::Info,
            search_term: String::new(),
        }
    }
}

impl LoggerUi {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Log");
        let mut logs = LOG.lock().unwrap();

        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                *logs = vec![]
            }

            egui::ComboBox::from_label("Log Level")
                .selected_text(self.loglevel.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.loglevel, log::Level::Info, "Info");
                    ui.selectable_value(&mut self.loglevel, log::Level::Warn, "Warn");
                    ui.selectable_value(&mut self.loglevel, log::Level::Error, "Error");
                });
        });
        ui.horizontal(|ui| {
            ui.label("Search: ");
            ui.text_edit_singleline(&mut self.search_term);
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            logs.iter().for_each(|(level, string)| {
                let string_format = format!("[{}]: {}", level, string);

                if !self.search_term.is_empty() && !string.contains(&self.search_term) {
                    return;
                }

                if &self.loglevel < level {
                    return;
                }

                match level {
                    log::Level::Warn => {
                        ui.colored_label(Color32::YELLOW, string_format);
                    }
                    log::Level::Error => {
                        ui.colored_label(Color32::RED, string_format);
                    }
                    _ => {
                        ui.label(string_format);
                    }
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
