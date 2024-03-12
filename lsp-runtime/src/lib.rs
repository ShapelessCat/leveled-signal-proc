pub type Timestamp = u64;
pub type Duration = u64;

pub mod context;
pub mod instrument;
pub mod signal_api;

mod moment;

pub use moment::Moment;
