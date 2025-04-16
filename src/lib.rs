#![doc = include_str!("../README.md")]
mod ui;

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

pub use ui::logger_ui;
pub use ui::LoggerUi;

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
}

impl EguiLogger {
    fn new(max_level: log::LevelFilter, show_all_categories: bool) -> Self {
        Self {
            max_level,
            show_all_categories,
        }
    }
}

/// The builder for the logger.
/// You can use [`builder()`] to get an instance of this.
pub struct Builder {
    max_level: log::LevelFilter,
    show_all_categories: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            max_level: log::LevelFilter::Debug,
            show_all_categories: true,
        }
    }
}

impl Builder {
    /// Returns the Logger.
    /// Useful if you want to add it to a multi-logger.
    /// See [here](https://github.com/RegenJacob/egui_logger/blob/main/examples/multi_log.rs) for an example.
    pub fn build(self) -> EguiLogger {
        EguiLogger::new(self.max_level, self.show_all_categories)
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
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Ok(ref mut logger) = LOGGER.lock() {
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
                    logger.max_category_length =
                        logger.max_category_length.max(record.target().len());
                }
            }
        }
    }

    fn flush(&self) {}
}

/// Initializes the global logger.
/// Should be called very early in the program.
/// Defaults to max level Debug.
///
/// This is now deprecated, use [`builder()`] instead.
#[deprecated(
    since = "0.5.0",
    note = "Please use `egui_logger::builder().init()` instead"
)]
pub fn init() -> Result<(), SetLoggerError> {
    builder().init()
}

/// Same as [`init()`] accepts a [`log::LevelFilter`] to set the max level
/// use [`Trace`](log::LevelFilter::Trace) with caution
///
/// This is now deprecated, use [`builder()`] instead.
#[deprecated(
    since = "0.5.0",
    note = "Please use `egui_logger::builder().max_level(max_level).init()` instead"
)]
pub fn init_with_max_level(max_level: log::LevelFilter) -> Result<(), SetLoggerError> {
    builder().max_level(max_level).init()
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
