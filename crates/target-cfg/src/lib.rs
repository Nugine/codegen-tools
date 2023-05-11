#![deny(clippy::all)]

mod ast;
mod eval;
mod fmt;
mod utils;

pub use self::ast::*;
pub use self::eval::*;
