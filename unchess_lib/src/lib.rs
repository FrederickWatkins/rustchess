//! Chess backend and engine library
#![allow(dead_code)]

pub mod error;
pub mod enums;
pub mod traits;

#[cfg(doctest)]
#[doc = include_str!("../../Readme.md")]
pub struct ReadmeDoctests;