mod duration;
mod peek;
pub mod combinator;

pub use duration::{DurationSinceBecomeTrue, DurationSinceLastLevel, DurationTrue, ScopedDurationTrue};
pub use peek::{DiffSinceCurrentLevel, Peek, PeekTimestamp};
