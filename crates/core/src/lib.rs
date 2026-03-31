pub mod add;
pub mod commands;
pub mod config;
pub mod errors;
pub mod filesystem;
pub mod git;
pub mod github;
pub mod init;
pub mod materialize;
pub mod output;
pub mod process;
pub mod setup;
pub mod state;

#[cfg(test)]
mod init_tests;

pub const PRODUCT_NAME: &str = "zazzles";
pub const CLI_NAME: &str = "zaz";
