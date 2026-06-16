pub mod instance;
pub mod macos_dock;
pub mod notifications;
pub mod tray;

pub use instance::{request_focus_if_running, start_server, InstanceServer};
pub use macos_dock::set_dock_visible;
pub use tray::{install as install_tray, poll_actions as poll_tray_actions, TrayAction};
