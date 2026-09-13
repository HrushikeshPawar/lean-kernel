mod debruijn;
mod reduction;
mod syntax;
mod typing;
mod universe;

pub use syntax::{LeanDefinition, LeanExpr, Level, Name};
pub use typing::TypeError;

#[cfg(test)]
mod tests;
