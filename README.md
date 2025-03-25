[![Crates.io](https://img.shields.io/crates/v/egui_logger)](https://crates.io/crates/egui_logger)
[![docs.rs](https://img.shields.io/docsrs/egui_logger)](https://docs.rs/egui_logger/latest/egui_logger/)



# egui_logger
This library implements a UI for displaying [`log`](https://crates.io/crates/log) messages in [`egui`](https://crates.io/crates/egui) applications.
There are also various ways to filter the logging output within the UI, such as a regex search through the messages.

## Demo
![demo](images/egui_logger.png "Demo")

## Example

### Initializing:
```rust
fn main() {
  // Should be called very early in the program.
  egui_logger::builder().init().unwrap();
}
```

### Inside your UI logic:
```rust
fn ui(ctx: &egui::Context) {
    egui::Window::new("Log").show(ctx, |ui| {
        // draws the logger ui.
        egui_logger::logger_ui().show(ui);
    });
}
```

## Alternatives
- [egui_tracing](https://crates.io/crates/egui_tracing) primarily for the [tracing](https://crates.io/crates/tracing) crate, but also supports log.

## Contribution
Feel free to open issues and pull requests.
