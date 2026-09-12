mod parser;
mod records;

pub use parser::parse_supported_record;
pub use records::*;

#[cfg(test)]
mod tests;
