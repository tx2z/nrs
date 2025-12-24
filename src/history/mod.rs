//! History module for nrs.
//!
//! Tracks script execution history per project for recent sorting
//! and quick rerun functionality.

mod storage;

pub use storage::{
    History, ProjectHistory, ScriptHistory, DEFAULT_MAX_PROJECTS, DEFAULT_MAX_SCRIPTS,
};
