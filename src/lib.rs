#![doc = include_str!("../README.md")]
mod ui;

use std::sync::Mutex;

pub use ui::logger_ui;
pub use ui::LoggerUi;

use log::SetLoggerError;
use ui::try_mut_log;

const LEVELS: [log::Level; log::Level::Trace as usize] = [
    log::Level::Error,
    log::Level::Warn,
    log::Level::Info,
    log::Level::Debug,
    log::Level::Trace,
];

/// The logger for egui
/// You might want to use [`builder()`] instead.
/// To get a builder with default values.
pub struct EguiLogger;

/// The builder for the logger.
/// You can use [`builder()`] to get an instance of this.
pub struct Builder {
    max_level: log::LevelFilter,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            max_level: log::LevelFilter::Debug,
        }
    }
}

impl Builder {
    /// Returns the Logger.
    /// Useful if you want to add it to a multi-logger.
    /// See [here](https://github.com/RegenJacob/egui_logger/blob/main/examples/multi_log.rs) for an example.
    pub fn build(self) -> EguiLogger {
        EguiLogger
    }

    /// Sets the max level for the logger
    /// this only has an effect when calling [`init()`].
    ///
    /// Defaults to [Debug](`log::LevelFilter::Debug`).
    pub fn max_level(mut self, max_level: log::LevelFilter) -> Self {
        self.max_level = max_level;
        self
    }

    /// Initializes the global logger.
    /// This should be called very early in the program.
    ///
    /// The max level is the [max_level](Self::max_level) field.
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_logger(&EguiLogger).map(|()| log::set_max_level(self.max_level))
    }
}

impl log::Log for EguiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::STATIC_MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            try_mut_log(|logs| {
                logs.push((
                    record.level(),
                    record.args().to_string(),
                    record.target().to_string(),
                ))
            });
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

pub(crate) type GlobalLog = Vec<(log::Level, String, String)>;

static LOG: Mutex<GlobalLog> = Mutex::new(Vec::new());

/**
This returns the Log builder with default values.
This is just a conveniend way to get call [`Builder::default()`].
[Read more](`crate::Builder`)

Example:
```rust
use log::LevelFilter;
fn main() -> {
    // initialize the logger.
    // You have to open the ui later within your egui context logic.
    // You should call this very early in the program.
    egui_logger::builder()
        .max_level(LevelFilter::Info) // defaults to Debug
        .init()
        .unwrap();

    // ...
}
```
*/
pub fn builder() -> Builder {
    Builder::default()
}
