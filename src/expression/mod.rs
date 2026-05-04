mod evaluator;
mod parser;
#[cfg(test)]
mod tests;
mod token;

pub use evaluator::{evaluate, evaluate_json};
