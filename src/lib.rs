#![doc = include_str!("../README.md")]
mod ui;

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

pub use ui::LoggerUi;
pub use ui::logger_ui;

use log::SetLoggerError;

const LEVELS: [log::Level; log::Level::Trace as usize] = [
    log::Level::Error,
    log::Level::Warn,
    log::Level::Info,
    log::Level::Debug,
    log::Level::Trace,
];

/// The logger for egui.
///
/// You might want to use [`builder()`] instead to get a builder with default values.
pub struct EguiLogger {
    /// The maximum log level that shall be collected.
    max_level: log::LevelFilter,
    /// Whether to show all categories by default (versus only those that are explicitly enabled).
    show_all_categories: bool,

    blacklisted: Vec<String>,
}

impl EguiLogger {
    fn new(
        max_level: log::LevelFilter,
        show_all_categories: bool,
        blacklisted: Vec<String>,
    ) -> Self {
        Self {
            max_level,
            show_all_categories,
            blacklisted,
        }
    }
}

/// The builder for the logger.
/// You can use [`builder()`] to get an instance of this.
pub struct Builder {
    max_level: log::LevelFilter,
    show_all_categories: bool,
    /// The default blacklist contains some `tracing` targets because they're just too fast for
    /// egui_logger
    blacklisted: Vec<String>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            max_level: log::LevelFilter::Debug,
            show_all_categories: true,
            blacklisted: vec![
                "tracing::span".to_string(),
                "tracing::span::active".to_string(),
            ],
        }
    }
}

impl Builder {
    /// Returns the Logger.
    /// Useful if you want to add it to a multi-logger.
    /// See [here](https://github.com/RegenJacob/egui_logger/blob/main/examples/multi_log.rs) for an example.
    pub fn build(self) -> EguiLogger {
        EguiLogger::new(self.max_level, self.show_all_categories, self.blacklisted)
    }

    /// Sets the max level for the logger.
    /// this only has an effect when calling [init](Self::init).
    ///
    /// Defaults to [Debug](`log::LevelFilter::Debug`).
    pub fn max_level(mut self, max_level: log::LevelFilter) -> Self {
        self.max_level = max_level;
        self
    }

    /// Whether to show all categories by default (versus only those that are explicitly enabled).
    ///
    /// Defaults to true.
    pub fn show_all_categories(mut self, show_all_categories: bool) -> Self {
        self.show_all_categories = show_all_categories;
        self
    }

    /// Whether or not the buildin blacklist is enabled.
    /// This just clears the blacklist so you should add custom rules after this.
    ///
    /// Defaults to true
    pub fn default_blacklist(mut self, default_blacklist: bool) -> Self {
        if default_blacklist {
            self
        } else {
            self.blacklisted = vec![];
            self
        }
    }

    /// This adds a `log` target to the blacklist.
    pub fn add_blacklist(mut self, target: impl ToString) -> Self {
        self.blacklisted.push(target.to_string());
        self
    }

    /// Initializes the global logger.
    /// This should be called very early in the program.
    ///
    /// The max level is the [max_level](Self::max_level) field.
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.max_level);
        log::set_logger(Box::leak(Box::new(self.build())))
    }
}

impl log::Log for EguiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.max_level
            && !self.blacklisted.contains(&metadata.target().to_string())
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata())
            && let Ok(ref mut logger) = LOGGER.lock()
        {
            logger.logs.push(Record {
                level: record.level(),
                message: record.args().to_string(),
                target: record.target().to_string(),
                time: chrono::Local::now(),
            });

            if !logger.categories.contains_key(record.target()) {
                logger
                    .categories
                    .insert(record.target().to_string(), self.show_all_categories);
                logger.max_category_length = logger.max_category_length.max(record.target().len());
            }
        }
    }

    fn flush(&self) {}
}

struct Record {
    level: log::Level,
    message: String,
    target: String,
    time: chrono::DateTime<chrono::Local>,
}

struct Logger {
    logs: Vec<Record>,
    categories: HashMap<String, bool>,
    max_category_length: usize,
    start_time: chrono::DateTime<chrono::Local>,
}
static LOGGER: LazyLock<Mutex<Logger>> = LazyLock::new(|| {
    Mutex::new(Logger {
        logs: Vec::new(),
        categories: HashMap::new(),
        max_category_length: 0,
        start_time: chrono::Local::now(),
    })
});

/// Clears all existing retained logs.
pub fn clear_logs() {
    LOGGER
        .lock()
        .expect("could not get access to logger")
        .logs
        .clear();
}

/// This returns the Log builder with default values.
/// This is just a convenient way to get [`Builder::default()`].
/// [Read more](`crate::Builder`)
///
/// Example:
/// ```rust
/// use log::LevelFilter;
/// # #[allow(clippy::needless_doctest_main)]
/// fn main() {
///     // Initialize the logger.
///     // You have to open the ui later within your egui context logic.
///     // You should call this very early in the program.
///     egui_logger::builder()
///         .max_level(LevelFilter::Info) // defaults to Debug
///         .init()
///         .unwrap();
///
///     // ...
/// }
/// ```
pub fn builder() -> Builder {
    Builder::default()
}
