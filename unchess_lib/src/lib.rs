//! Chess backend and engine library
#![allow(dead_code)]

pub mod board;
pub mod default_types;
pub mod enums;
pub mod error;
mod fen;
pub mod notation;
mod pgn;
pub mod traits;

#[cfg(doctest)]
#[doc = include_str!("../../Readme.md")]
pub struct ReadmeDoctests;
