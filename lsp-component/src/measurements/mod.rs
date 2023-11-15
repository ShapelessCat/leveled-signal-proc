mod duration;
mod peek;
pub mod combinator;

pub use duration::{DurationSinceBecomeTrue, DurationTrue , ScopedDurationTrue, DurationSinceLastLevel};
pub use peek::{Peek, DiffSinceCurrentLevel};

