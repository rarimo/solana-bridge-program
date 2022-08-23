#![feature(destructuring_assignment)]
pub mod state;
pub mod instruction;
pub mod entrypoint;
pub mod processor;
pub mod error;
mod util;
mod instruction_validation;
mod merkle_node;