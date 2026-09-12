mod debruijn;
mod syntax;
mod typing;

pub use syntax::{LeanDefinition, LeanExpr, Level, Name};

#[cfg(test)]
mod tests;
