use std::sync::Mutex;

use egui::{Align, Color32, FontSelection, RichText, Style, text::LayoutJob};
use regex::{Regex, RegexBuilder};

use crate::{LEVELS, LOGGER, Logger, Record};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimePrecision {
    Seconds,
    Milliseconds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimeFormat {
    Utc,
    LocalTime,
    SinceStart,
    Hide,
}

struct LoggerStyle {
    enable_regex: bool,
    enable_ctx_menu: bool,
    enable_log_count: bool,
    enable_copy_button: bool,
    enable_search: bool,
    enable_max_log_output: bool,
    enable_levels_button: bool,
    enable_categories_button: bool,
    enable_time_button: bool,
    time_precision: TimePrecision,
    show_target: bool,
    time_format: TimeFormat,
    include_target: bool,
    include_level: bool,

    warn_color: Color32,
    error_color: Color32,
    highlight_color: Color32,
}

impl Default for LoggerStyle {
    fn default() -> Self {
        Self {
            show_target: true,
            enable_regex: true,
            enable_ctx_menu: true,
            include_target: true,
            include_level: true,
            time_format: TimeFormat::LocalTime,
            time_precision: TimePrecision::Seconds,
            warn_color: Color32::YELLOW,
            error_color: Color32::RED,
            highlight_color: Color32::LIGHT_GRAY,
            enable_log_count: true,
            enable_copy_button: true,
            enable_search: true,
            enable_max_log_output: true,
            enable_levels_button: true,
            enable_categories_button: true,
            enable_time_button: true,
        }
    }
}

/// The Ui for the Logger.
/// You can use [`logger_ui()`] to get a default instance of the LoggerUi.
pub struct LoggerUi {
    loglevels: [bool; log::Level::Trace as usize],
    search_term: String,
    regex: Option<Regex>,
    search_case_sensitive: bool,
    search_use_regex: bool,
    max_log_length: usize,
    style: LoggerStyle,
    /// Cached search results: true if record matches current search
    search_cache: Vec<bool>,
    /// Cached LayoutJobs for each log record
    layout_cache: Vec<LayoutJob>,
    /// Whether to cache LayoutJobs (more memory footprint but 30% more performant)
    cache_layouts: bool,
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
            style: LoggerStyle::default(),
            search_cache: Vec::new(),
            layout_cache: Vec::new(),
            cache_layouts: true,
        }
    }
}

impl LoggerUi {
    /// Enable or disable the regex search.
    /// True by default.
    #[inline] // i think the compiler already does this
    pub fn enable_regex(mut self, enable: bool) -> Self {
        self.style.enable_regex = enable;
        self
    }

    /// Enable or disable the context menu.
    /// True by default.
    #[inline]
    pub fn enable_ctx_menu(mut self, enable: bool) -> Self {
        self.style.enable_ctx_menu = enable;
        self
    }

    /// Enable or disable showing the [target](log::Record::target()) in the context menu.
    /// True by default.
    #[inline]
    pub fn show_target(mut self, enable: bool) -> Self {
        self.style.show_target = enable;
        self
    }

    /// Enable or disable showing the [target](log::Record::target()) in the records.
    /// True by default.
    #[inline]
    pub fn include_target(mut self, enable: bool) -> Self {
        self.style.include_target = enable;
        self
    }

    /// Enable or disable showing the [level](log::Record::level) in the records.
    /// True by default.
    #[inline]
    pub fn include_level(mut self, enable: bool) -> Self {
        self.style.include_level = enable;
        self
    }

    /// Enable or disable the copy button.
    /// True by default.
    #[inline]
    pub fn enable_copy_button(mut self, enable: bool) -> Self {
        self.style.enable_copy_button = enable;
        self
    }

    /// Enable or disable the count of how many log messages there are.
    /// True by default.
    #[inline]
    pub fn enable_log_count(mut self, enable: bool) -> Self {
        self.style.enable_log_count = enable;
        self
    }

    /// Enable or disable the count of how many log messages there are.
    /// True by default.
    #[inline]
    pub fn enable_search(mut self, enable: bool) -> Self {
        self.style.enable_search = enable;
        self
    }

    /// Enable or disable the configurable field for the maximum number of shown log output messages.
    /// True by default.
    #[inline]
    pub fn enable_max_log_output(mut self, enable: bool) -> Self {
        self.style.enable_max_log_output = enable;
        self
    }

    /// Enable or disable the button to configure the log levels.
    /// True by default.
    #[inline]
    pub fn enable_levels_button(mut self, enable: bool) -> Self {
        self.style.enable_levels_button = enable;
        self
    }

    /// Enable or disable the button to configure the log categories.
    /// True by default.
    #[inline]
    pub fn enable_categories_button(mut self, enable: bool) -> Self {
        self.style.enable_categories_button = enable;
        self
    }

    /// Enable or disable the button to configure the time format.
    /// True by default.
    #[inline]
    pub fn enable_time_button(mut self, enable: bool) -> Self {
        self.style.enable_time_button = enable;
        self
    }

    /// Set the color for warning messages.
    #[inline]
    pub fn warn_color(mut self, color: Color32) -> Self {
        self.style.warn_color = color;
        self
    }

    /// Set the color for error messages.
    #[inline]
    pub fn error_color(mut self, color: Color32) -> Self {
        self.style.error_color = color;
        self
    }

    /// Set the color for log messages that are neither errors nor warnings.
    #[inline]
    pub fn highlight_color(mut self, color: Color32) -> Self {
        self.style.highlight_color = color;
        self
    }

    /// Set which log levels should be enabled.
    /// The `log_levels` are specified as a boolean array where the first element
    /// corresponds to the `ERROR` level and the last one to the `TRACE` level.
    #[inline]
    pub fn log_levels(mut self, log_levels: [bool; log::Level::Trace as usize]) -> Self {
        self.loglevels = log_levels;
        self
    }

    /// Set which log levels should be enabled.
    ///
    /// # Panics
    /// Panics if the lock to the logger could not be acquired.
    #[inline]
    pub fn enable_category(self, category: impl ToString, enable: bool) -> Self {
        LOGGER
            .lock()
            .as_mut()
            .expect("could not lock LOGGER")
            .categories
            .insert(category.to_string(), enable);
        self
    }

    /// Set the maximum number of log messages that should be retained.
    #[inline]
    pub fn max_log_length(mut self, max_length: usize) -> Self {
        self.max_log_length = max_length;
        self
    }

    /// Enable or disable caching of formatted log lines.
    /// When enabled, increases memory usage but improves rendering performance.
    /// True by default.
    pub fn enable_cache_layouts(mut self, enable: bool) -> Self {
        self.cache_layouts = enable;
        self
    }

    pub(crate) fn log_ui(self) -> &'static Mutex<LoggerUi> {
        static LOGGER_UI: std::sync::OnceLock<Mutex<LoggerUi>> = std::sync::OnceLock::new();
        LOGGER_UI.get_or_init(|| self.into())
    }

    /// This draws the Logger UI.
    pub fn show(self, ui: &mut egui::Ui) {
        if let Ok(ref mut logger_ui) = self.log_ui().lock() {
            logger_ui.ui(ui);
        } else {
            ui.colored_label(Color32::RED, "Something went wrong loading the log");
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        #[cfg(feature = "puffin")]
        puffin::profile_scope!("render logger UI");
        let Ok(ref mut logger) = LOGGER.lock() else {
            return;
        };

        let dropped_entries = logger.logs.len().saturating_sub(self.max_log_length);
        drop(logger.logs.drain(..dropped_entries));

        // Sync cache with drained logs - remove stale entries from front
        // New logs will be appended later depending on the search ui response.
        if dropped_entries > 0 {
            let drain_count = dropped_entries.min(self.search_cache.len());
            drop(self.search_cache.drain(..drain_count));
            if self.cache_layouts {
                let layout_drain = dropped_entries.min(self.layout_cache.len());
                drop(self.layout_cache.drain(..layout_drain));
            }
        }

        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                logger.logs.clear();
                self.search_cache.clear();
                self.layout_cache.clear();
            }

            if self.style.enable_levels_button {
                ui.menu_button("Log Levels", |ui| {
                    for level in LEVELS {
                        if ui
                            .selectable_label(self.loglevels[level as usize - 1], level.as_str())
                            .clicked()
                        {
                            self.loglevels[level as usize - 1] =
                                !self.loglevels[level as usize - 1];
                        }
                    }
                });
            }

            if self.style.enable_categories_button {
                ui.menu_button("Categories", |ui| {
                    if ui.button("Select All").clicked() {
                        for (_, enabled) in logger.categories.iter_mut() {
                            *enabled = true;
                        }
                    }

                    if ui.button("Unselect All").clicked() {
                        for (_, enabled) in logger.categories.iter_mut() {
                            *enabled = false;
                        }
                    }

                    for (category, enabled) in logger.categories.iter_mut() {
                        if ui.selectable_label(*enabled, category).clicked() {
                            *enabled = !*enabled;
                        }
                    }
                });
            }

            if self.style.enable_time_button {
                ui.menu_button("Time", |ui| {
                    ui.radio_value(&mut self.style.time_format, TimeFormat::Utc, "UTC");
                    ui.radio_value(
                        &mut self.style.time_format,
                        TimeFormat::LocalTime,
                        "Local Time",
                    );
                    ui.radio_value(
                        &mut self.style.time_format,
                        TimeFormat::SinceStart,
                        "Since Start",
                    );
                    ui.radio_value(&mut self.style.time_format, TimeFormat::Hide, "Hide");

                    ui.separator();

                    ui.radio_value(
                        &mut self.style.time_precision,
                        TimePrecision::Seconds,
                        "Seconds",
                    );
                    ui.radio_value(
                        &mut self.style.time_precision,
                        TimePrecision::Milliseconds,
                        "Milliseconds",
                    );
                });
            }
        });

        let mut search_changed = false;
        if self.style.enable_search {
            ui.horizontal(|ui| {
                ui.label("Search: ");
                let response = ui.text_edit_singleline(&mut self.search_term);

                if response.changed() {
                    search_changed = true;
                }

                if ui
                    .selectable_label(self.search_case_sensitive, "Aa")
                    .on_hover_text("Case sensitive")
                    .clicked()
                {
                    self.search_case_sensitive = !self.search_case_sensitive;
                    search_changed = true;
                }

                if self.style.enable_regex
                    && ui
                        .selectable_label(self.search_use_regex, ".*")
                        .on_hover_text("Use regex")
                        .clicked()
                {
                    self.search_use_regex = !self.search_use_regex;
                    search_changed = true;
                }

                if self.style.enable_regex && self.search_use_regex && search_changed {
                    self.regex = RegexBuilder::new(&self.search_term)
                        .case_insensitive(!self.search_case_sensitive)
                        .build()
                        .ok()
                }
            });
        }

        if self.style.enable_max_log_output {
            ui.horizontal(|ui| {
                ui.label("Max Log output");
                ui.add(egui::widgets::DragValue::new(&mut self.max_log_length).speed(1));
            });
        }

        ui.separator();

        let time_padding = logger.logs.last().map_or(0, |record| {
            format_time(record.time, &self.style, logger.start_time).len()
        });

        // Add new records to the cache layout if enabled.
        if self.cache_layouts {
            for record in logger.logs.iter().skip(self.layout_cache.len()) {
                self.layout_cache
                    .push(format_record(logger, &self.style, record, time_padding));
            }
        }

        // Update search cache with new records, or rebuilds it if search content changed.
        self.update_search_cache(logger, time_padding, search_changed);

        // Pre-filter by level, category, and cached search result
        let filtered_logs: Vec<usize> = logger
            .logs
            .iter()
            .enumerate()
            .filter(|(_, r)| self.loglevels[r.level as usize - 1])
            .filter(|(_, record)| !matches!(logger.categories.get(&record.target), Some(false)))
            .filter(|(i, _)| self.search_cache.get(*i).copied().unwrap_or(true))
            .map(|(i, _)| i)
            .collect();

        let logs_displayed = filtered_logs.len();

        let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(ui.available_height() - 30.0)
            .show_rows(ui, row_height, logs_displayed, |ui, row_range| {
                for i in row_range {
                    let log_idx = filtered_logs[i];
                    let record = &logger.logs[log_idx];
                    let layout_job = if self.cache_layouts {
                        &self.layout_cache[log_idx]
                    } else {
                        &format_record(logger, &self.style, record, time_padding)
                    };

                    let response = ui.label(layout_job.clone());

                    if self.style.enable_ctx_menu {
                        response.clone().context_menu(|ui| {
                            if self.style.show_target {
                                ui.label(&record.target);
                            }
                            response.highlight();
                            let string_format = format!("[{}]: {}", record.level, record.message);

                            ui.vertical(|ui| {
                                ui.monospace(string_format);
                            });

                            if ui.button("Copy").clicked() {
                                ui.ctx().copy_text(layout_job.text.clone());
                            }
                        });
                    }
                }
            });

        ui.horizontal(|ui| {
            if self.style.enable_log_count {
                ui.label(format!("Log size: {}", logger.logs.len()));
                ui.label(format!("Displayed: {}", logs_displayed));
            }
            if self.style.enable_copy_button {
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Copy").clicked() {
                        let mut out_string = String::new();
                        logger
                            .logs
                            .iter()
                            .take(self.max_log_length)
                            .for_each(|record| {
                                out_string.push_str(
                                    &format_record(logger, &self.style, record, time_padding).text,
                                );
                                out_string.push_str(" \n");
                            });
                        ui.ctx().copy_text(out_string);
                    }
                });
            }
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

    fn update_search_cache(&mut self, logger: &Logger, time_padding: usize, full_rebuild: bool) {
        let start = if full_rebuild {
            self.search_cache.clear();
            self.search_cache.reserve(logger.logs.len());
            0
        } else {
            self.search_cache.len()
        };

        let new_count = logger.logs.len() - start;
        if new_count == 0 {
            return;
        }

        if self.search_term.is_empty() {
            self.search_cache
                .extend(std::iter::repeat_n(true, new_count));
            return;
        }

        // Use layout cache if available, otherwise format each record
        if self.cache_layouts && self.layout_cache.len() == logger.logs.len() {
            for layout in self.layout_cache.iter().skip(start) {
                self.search_cache.push(self.match_string(&layout.text));
            }
        } else {
            for record in logger.logs.iter().skip(start) {
                let text = format_record(logger, &self.style, record, time_padding).text;
                self.search_cache.push(self.match_string(&text));
            }
        }
    }
}

/// Returns a default LoggerUi.
/// You have to call [`LoggerUi::show()`] to display the logger.
pub fn logger_ui() -> LoggerUi {
    LoggerUi::default()
}

fn format_time(
    time: chrono::DateTime<chrono::Local>,
    style: &LoggerStyle,
    start_time: chrono::DateTime<chrono::Local>,
) -> String {
    let time = match (style.time_format, style.time_precision) {
        (TimeFormat::Utc, TimePrecision::Seconds) => time
            .to_utc()
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        (TimeFormat::Utc, TimePrecision::Milliseconds) => time
            .to_utc()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        (TimeFormat::LocalTime, TimePrecision::Seconds) => time.format("%T").to_string(),
        (TimeFormat::LocalTime, TimePrecision::Milliseconds) => time.format("%T%.3f").to_string(),
        (TimeFormat::SinceStart, TimePrecision::Seconds) => {
            let duration = time - start_time;
            let h = duration.num_hours() % 24;
            let m = duration.num_minutes() % 60;
            let s = duration.num_seconds() % 60;
            match (h, m, s) {
                (0, 0, s) => format!("{s}s"),
                (0, m, s) => format!("{m}m {s}s"),
                (h, m, s) => format!("{h}h {m}m {s}s"),
            }
        }
        (TimeFormat::SinceStart, TimePrecision::Milliseconds) => {
            let duration = time - start_time;
            let h = duration.num_hours() % 24;
            let m = duration.num_minutes() % 60;
            let s = duration.num_seconds() % 60;
            let ms = duration.num_milliseconds() % 1000;
            match (h, m, s, ms) {
                (0, 0, 0, ms) => format!("{ms}ms"),
                (0, 0, s, ms) => format!("{s}s {ms}ms"),
                (0, m, s, ms) => format!("{m}m {s}s {ms}ms"),
                (h, m, s, ms) => format!("{h}h {m}m {s}s {ms}ms"),
            }
        }
        (TimeFormat::Hide, _) => String::new(),
    };
    if style.time_format == TimeFormat::Hide {
        time
    } else {
        time + " "
    }
}

fn format_record(
    logger: &Logger,
    logger_style: &LoggerStyle,
    record: &Record,
    time_padding: usize,
) -> LayoutJob {
    let level_str = if logger_style.include_level {
        format!("[{:5}] ", record.level)
    } else {
        String::new()
    };
    let target_str = if logger_style.include_target {
        format!(
            "{: <width$}: ",
            record.target,
            width = logger.max_category_length
        )
    } else {
        String::new()
    };
    let mut layout_job = LayoutJob::default();
    let style = Style::default();

    let mut date_str = RichText::new(format!(
        "{: >width$}",
        format_time(record.time, logger_style, logger.start_time),
        width = time_padding
    ))
    .monospace();
    match record.level {
        log::Level::Warn => date_str = date_str.color(logger_style.warn_color),
        log::Level::Error => date_str = date_str.color(logger_style.error_color),
        _ => {}
    }

    date_str.append_to(&mut layout_job, &style, FontSelection::Default, Align::LEFT);

    let highlight_color = match record.level {
        log::Level::Warn => logger_style.warn_color,
        log::Level::Error => logger_style.error_color,
        _ => logger_style.highlight_color,
    };

    RichText::new(level_str + &target_str)
        .monospace()
        .color(highlight_color)
        .append_to(&mut layout_job, &style, FontSelection::Default, Align::LEFT);

    let mut message = RichText::new(&record.message).monospace();
    match record.level {
        log::Level::Warn => message = message.color(logger_style.warn_color),
        log::Level::Error => message = message.color(logger_style.error_color),
        _ => {}
    }

    message.append_to(&mut layout_job, &style, FontSelection::Default, Align::LEFT);

    layout_job
}
