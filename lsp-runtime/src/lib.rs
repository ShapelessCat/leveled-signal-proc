pub type Timestamp = u64;
pub type Duration = u64;

mod context;
mod internal_queue;
mod moment;
mod multipeek;

pub use context::{InputSignalBag, LspContext, UpdateContext, WithTimestamp};
pub use internal_queue::InternalEventQueue;
pub use moment::Moment;
pub use multipeek::MultiPeek;

pub mod instrument;
pub mod signal_api;
