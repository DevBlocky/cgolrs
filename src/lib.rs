//! Core library for Conway's Game of Life.

pub mod enc;
pub mod engine;
pub mod pos;

pub use enc::{Codec, RunLengthEncoded};
pub use engine::{GameEngineWindow, GameOfLife};
pub use pos::Pos2;
