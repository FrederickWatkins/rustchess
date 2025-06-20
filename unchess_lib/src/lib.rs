//! Chess backend and engine library

pub mod board;
pub mod enums;
pub mod error;
pub mod notation;
mod parser;
pub mod simple_types;
pub mod traits;

#[cfg(doctest)]
#[doc = include_str!("../../Readme.md")]
pub struct ReadmeDoctests;
