// src/compression/mod.rs — Compression sub-modules (Gorilla, RLE, Varint, Bit I/O)

pub mod bitio;
pub mod gorilla;
pub mod rle;
pub mod varint;
#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility
#[allow(unused_imports)]
pub use bitio::*;
#[allow(unused_imports)]
pub use gorilla::*;
#[allow(unused_imports)]
pub use rle::*;
#[allow(unused_imports)]
pub use varint::*;