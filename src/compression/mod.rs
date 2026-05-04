// src/compression/mod.rs — Compression sub-modules (Gorilla, RLE, Varint, Bit I/O)

pub mod bitio;
pub mod gorilla;
pub mod rle;
pub mod varint;
#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility
pub use bitio::*;
pub use gorilla::*;
pub use rle::*;
pub use varint::*;
