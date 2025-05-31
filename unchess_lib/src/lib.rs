#![allow(dead_code)]

pub mod board;
pub mod error;
pub mod traits;
pub mod types;
pub mod game;
pub mod engine;
pub mod piece;

pub use traits::*;

#[cfg(doctest)]
#[doc = include_str!("../../Readme.md")]
pub struct ReadmeDoctests;
