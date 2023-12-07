pub mod combinator;
mod duration;
mod peek;

pub use duration::{
    DurationSinceBecomeTrue, DurationSinceLastLevel, DurationTrue, ScopedDurationTrue,
};
pub use peek::{DiffSinceCurrentLevel, Peek, PeekTimestamp};
