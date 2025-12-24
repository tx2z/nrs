//! Integration tests for nrs.
//!
//! This module contains comprehensive integration tests organized by feature:
//!
//! - `fixtures` - Test helpers for creating temporary projects
//! - `cli_tests` - CLI interface tests
//! - `detection_tests` - Package manager detection tests
//! - `config_tests` - Configuration loading and merging tests
//! - `history_tests` - History recording and retrieval tests
//! - `snapshot_tests` - Output snapshot tests using insta

pub mod cli_tests;
pub mod config_tests;
pub mod detection_tests;
pub mod fixtures;
pub mod history_tests;
pub mod snapshot_tests;
