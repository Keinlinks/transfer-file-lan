#![warn(clippy::all, rust_2018_idioms)]
pub mod mdns_module;
mod confirm_window;
pub use confirm_window::ConfirmWindow;
pub mod structs;
mod download_progress_window;
pub use download_progress_window::DownloadWindow;
mod send_to_windows;
pub use send_to_windows::SendWindow;