#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
pub mod webkitgtk;

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
pub use webkitgtk::{headless, windowed};

// #[cfg(target_os = "windows")]
// pub mod webview2;

// #[cfg(target_os = "windows")]
// pub use webview2::{headless, windowed};
