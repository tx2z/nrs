//! Custom widgets for the TUI.
//!
//! This module contains specialized widgets for rendering the nrs interface.

mod description;
mod filter;
mod footer;
mod header;
mod scripts;

pub use description::{Description, ErrorDisplay};
pub use filter::{ArgsFilter, Filter};
pub use footer::{Footer, MessageFooter};
pub use header::{truncate_with_ellipsis, Header};
pub use scripts::{EmptyScripts, ScriptsGrid};
