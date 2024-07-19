use std::sync::Mutex;

use egui::Color32;
use regex::{Regex, RegexBuilder};

use crate::{GlobalLog, LEVELS, LOG};

pub(crate) fn try_mut_log<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&mut GlobalLog) -> T,
{
    match LOG.lock() {
        Ok(ref mut global_log) => Some((f)(global_log)),
        Err(_) => None,
    }
}

fn try_get_log<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&GlobalLog) -> T,
{
    match LOG.lock() {
        Ok(ref global_log) => Some((f)(global_log)),
        Err(_) => None,
    }
}

struct Style {
    enable_regex: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self { enable_regex: true }
    }
}

/// The Ui for the Logger.
/// You can use [`logger_ui()`] to get a default instance of the LoggerUi
pub struct LoggerUi {
    loglevels: [bool; log::Level::Trace as usize],
    search_term: String,
    regex: Option<Regex>,
    search_case_sensitive: bool,
    search_use_regex: bool,
    max_log_length: usize,
    style: Style,
}

impl Default for LoggerUi {
    fn default() -> Self {
        Self {
            loglevels: [true, true, true, false, false],
            search_term: String::new(),
            search_case_sensitive: false,
            regex: None,
            search_use_regex: false,
            max_log_length: 1000,
            style: Style::default(),
        }
    }
}

impl LoggerUi {
    /// Enable or disable the regex search
    /// Default is true
    pub fn enable_regex(mut self, enable: bool) -> Self {
        self.style.enable_regex = enable;
        self
    }

    pub(crate) fn log_ui(self) -> &'static Mutex<LoggerUi> {
        static LOGGER_UI: std::sync::OnceLock<Mutex<LoggerUi>> = std::sync::OnceLock::new();
        LOGGER_UI.get_or_init(|| self.into())
    }

    /// This draws the Logger UI
    pub fn show(self, ui: &mut egui::Ui) {
        if let Ok(ref mut logger_ui) = self.log_ui().lock() {
            logger_ui.ui(ui);
        } else {
            ui.colored_label(Color32::RED, "Something went wrong loading the log");
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        try_mut_log(|logs| {
            let dropped_entries = logs.len().saturating_sub(self.max_log_length);
            drop(logs.drain(..dropped_entries));
        });

        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                try_mut_log(|logs| logs.clear());
            }
            ui.menu_button("Log Levels", |ui| {
                for level in LEVELS {
                    if ui
                        .selectable_label(self.loglevels[level as usize - 1], level.as_str())
                        .clicked()
                    {
                        self.loglevels[level as usize - 1] = !self.loglevels[level as usize - 1];
                    }
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label("Search: ");
            let response = ui.text_edit_singleline(&mut self.search_term);

            let mut config_changed = false;

            if ui
                .selectable_label(self.search_case_sensitive, "Aa")
                .on_hover_text("Case sensitive")
                .clicked()
            {
                self.search_case_sensitive = !self.search_case_sensitive;
                config_changed = true;
            }

            if self.style.enable_regex
                && ui
                    .selectable_label(self.search_use_regex, ".*")
                    .on_hover_text("Use regex")
                    .clicked()
            {
                self.search_use_regex = !self.search_use_regex;
                config_changed = true;
            }

            if self.style.enable_regex
                && self.search_use_regex
                && (response.changed() || config_changed)
            {
                self.regex = RegexBuilder::new(&self.search_term)
                    .case_insensitive(!self.search_case_sensitive)
                    .build()
                    .ok()
            }
        });

        ui.horizontal(|ui| {
            ui.label("Max Log output");
            ui.add(egui::widgets::DragValue::new(&mut self.max_log_length).speed(1));
        });

        ui.horizontal(|ui| {
            if ui.button("Sort").clicked() {
                try_mut_log(|logs| logs.sort());
            }
        });

        ui.separator();

        let mut logs_displayed: usize = 0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .max_height(ui.available_height() - 30.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                try_get_log(|logs| {
                    logs.iter().for_each(|(level, string)| {
                        if (!self.search_term.is_empty() && !self.match_string(string))
                            || !(self.loglevels[*level as usize - 1])
                        {
                            return;
                        }

                        let string_format = format!("[{}]: {}", level, string);

                        let response = match level {
                            log::Level::Warn => ui.colored_label(Color32::YELLOW, string_format),
                            log::Level::Error => ui.colored_label(Color32::RED, string_format),
                            _ => ui.label(string_format),
                        };

                        response.clone().context_menu(|ui| {
                            response.highlight();
                            match level {
                                log::Level::Warn => {
                                    ui.colored_label(Color32::YELLOW, "WARNING:")
                                }
                                log::Level::Error => {
                                    ui.colored_label(Color32::RED, "ERROR:")
                                }
                                _ => ui.label(format!("{level}:"))
                            };

                            if ui.button("Copy").clicked() {
                                ui.output_mut(|o| {
                                    o.copied_text = string.to_string();
                                });
                            }
                        });

                        logs_displayed += 1;
                    });
                });
            });

        ui.horizontal(|ui| {
            ui.label(format!(
                "Log size: {}",
                try_get_log(|logs| logs.len()).unwrap_or_default()
            ));
            ui.label(format!("Displayed: {}", logs_displayed));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Copy").clicked() {
                    ui.output_mut(|o| {
                        try_get_log(|logs| {
                            let mut out_string = String::new();
                            logs.iter()
                                .take(self.max_log_length)
                                .for_each(|(_, string)| {
                                    out_string.push_str(string);
                                    out_string.push_str(" \n");
                                });
                            o.copied_text = out_string;
                        });
                    });
                }
            });
        });
    }

    fn match_string(&self, string: &str) -> bool {
        if self.search_use_regex {
            if let Some(matcher) = &self.regex {
                matcher.is_match(string)
            } else {
                false
            }
        } else if self.search_case_sensitive {
            string.contains(&self.search_term)
        } else {
            string
                .to_lowercase()
                .contains(&self.search_term.to_lowercase())
        }
    }
}

/// Returns a default LoggerUi.
/// You have to call [`LoggerUi::show()`] to display the logger
pub fn logger_ui() -> LoggerUi {
    LoggerUi::default()
}
