mod accmulator;
mod duration;
mod generator;
mod latch;
mod liveness;
mod mapper;
mod state_machine;

pub use accmulator::Accumulator;
pub use duration::DurationOfPreviousLevel;
pub use generator::SignalGenerator;
pub use latch::Latch;
pub use liveness::LivenessChecker;
pub use mapper::SignalMapper;
pub use state_machine::StateMachine;

#[cfg(test)]
pub(crate) mod test {
    use lsp_runtime::{Timestamp, WithTimestamp, InputSignalBag, LspContext};

    #[derive(Default, Clone)]
    pub(crate) struct TestSignalBag {
        pub(crate) value: u32,
    }
    #[derive(Default, Clone)]
    pub(crate) struct TestSignalInput {
        pub(crate) value: u32,
        pub(crate) timestamp: Timestamp,
    }

    impl WithTimestamp for TestSignalInput {
        fn timestamp(&self) -> u64 {
            self.timestamp
        }
    }

    impl InputSignalBag for TestSignalBag {
        type Input = TestSignalInput;

        fn patch(&mut self, patch: Self::Input) {
            self.value = patch.value;
        }
    }

    pub(crate) fn create_lsp_context_for_test() -> LspContext<std::vec::IntoIter<TestSignalInput>, TestSignalBag> {
        LspContext::new((0..1000).map(|i| TestSignalInput {
            value: i as u32,
            timestamp: i,
        }).collect::<Vec<_>>().into_iter())
    }
}