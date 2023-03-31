#![feature(destructuring_assignment)]
#![feature(array_methods)]
pub mod state;
pub mod instruction;
pub mod entrypoint;
pub mod processor;
pub mod error;
mod util;
mod instruction_validation;
mod merkle;
mod commission;