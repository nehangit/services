// TODO remove this once the crate stabilizes a bit.
#![allow(dead_code)]
#![recursion_limit = "256"]

pub mod api;
mod boundary;
pub mod domain;
mod infra;
mod run;
#[cfg(test)]
mod tests;
mod util;

pub use self::run::{run, start};
