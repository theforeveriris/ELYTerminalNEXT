mod blocks;
mod boxes;
mod click;
pub(crate) mod device;
mod drop;
mod find;
mod fonts;
pub mod frame;
mod herdr;
pub mod ink;
pub mod input;
pub mod keys;
mod links;
#[cfg(target_os = "macos")]
pub mod mac;
mod marks;
pub mod mouse;
mod notify;
mod paint;
mod paste;
mod pointer;
mod prompts;
pub(crate) mod pty;
mod reports;
mod scrollbar;
pub mod shell;
mod status;
mod tap;
mod titles;
#[cfg(unix)]
mod unix;
mod view;
#[cfg(windows)]
mod win32;
pub mod zoom;

pub use blocks::{Block, Blocks, Phase};
pub use drop::{Quote, Target};
pub use fonts::{ICONS, MONO, load_font};
pub use pty::{
    Events, Failed, Launch, Listener, Pty, PtyEvent, Started, config, failed, fit, opened, spawn,
};
pub use status::{Kind, Record, Records, State};
pub use view::{BLINK, TerminalEvent, TerminalView};
