#![allow(dead_code)]

pub mod board;
pub mod error;
pub mod traits;
pub mod types;
pub mod game;
mod piece;

pub use traits::*;

#[cfg(doctests)]
#[doc = include_str!("../../Readme.md")]
pub struct ReadmeDoctests;
