#![feature(box_patterns)]

#[cfg(feature = "binary-ir")]
pub mod binary;

pub mod ssa;
pub mod tree;

pub mod compiler;
pub mod linker;
pub mod target;
pub mod types;

pub mod style;
