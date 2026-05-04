mod token;
mod parser;
mod evaluator;
#[cfg(test)]
mod tests;

pub use evaluator::{evaluate, evaluate_json};
