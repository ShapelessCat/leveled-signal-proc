pub mod combinator;
mod duration;
mod linear_change;
mod peek;

pub use duration::{
    DurationSinceBecomeTrue, DurationSinceLastLevel, DurationTrue, ScopedDurationTrue,
};
pub use linear_change::{LinearChange, ScopedLinearChange};
pub use peek::{DiffSinceCurrentLevel, Peek, PeekTimestamp};
