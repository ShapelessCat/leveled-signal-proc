pub mod combinator;
mod duration;
mod linear_change;
mod peek;
// TODO: Remove all ScopedXXXX type alias after adding direct measurement combinator support in LSDL and Codegen!
pub use duration::{
    DurationSinceBecomeTrue, DurationSinceLastLevel, DurationTrue, ScopedDurationTrue,
};
pub use linear_change::{LinearChange, ScopedLinearChange};
pub use peek::{DiffSinceCurrentLevel, MappedPeekTimestamp, Peek, PeekTimestamp};
