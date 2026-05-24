#![feature(box_patterns)]

#[cfg(feature = "binary-ir")]
pub mod binary;

pub mod compiler;
pub mod ssa;
pub mod target;
pub mod tree;
pub mod types;
