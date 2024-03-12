use crate::Timestamp;

/// Some type with timestamp information.
/// Typically, an event taken from outside should implement this trait and the context is
/// responsible assemble the simultaneous event into the global input state.
pub trait WithTimestamp {
    fn timestamp(&self) -> Timestamp;
}

/// The global input state which applies the incoming events as patches to the state and this is the
/// external input type of the LSP system.
pub trait InputSignalBag: Clone + Default {
    type Input;
    /// Patch an event to the state
    fn patch(&mut self, patch: Self::Input);

    /// Determine if an input state need to trigger a measurement
    fn should_measure(&mut self) -> bool {
        false
    }
}
