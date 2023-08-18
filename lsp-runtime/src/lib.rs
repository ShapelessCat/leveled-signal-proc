pub type Timestamp = u64;

mod context;
mod internal_queue;
mod moment;
mod multipeek;

pub use context::{InputSignalBag, LspContext, UpdateContext, WithTimestamp};
pub use internal_queue::InternalEventQueue;
pub use moment::Moment;
pub use multipeek::MultiPeek;

pub mod measurement;
pub mod signal;

pub trait LspInstructmentationContext : Default {
    fn data_logic_exec_start(&mut self) {}
    fn data_logic_exec_end(&mut self) {}
}

#[derive(Default)]
pub struct NoInstrumentation;

impl LspInstructmentationContext for NoInstrumentation {}