mod debruijn;
mod reduction;
mod syntax;
mod typing;
mod universe;

pub use syntax::{LeanAxiom, LeanDeclaration, LeanDefinition, LeanExpr, LeanTheorem, Level, Name};
pub use typing::TypeError;

#[cfg(test)]
mod tests;
