pub mod measurements;
pub mod processors;

#[cfg(test)]
pub(crate) mod test {
    use std::vec::IntoIter;

    use lsp_runtime::context::{InputSignalBag, LspContext, WithTimestamp};
    use lsp_runtime::Timestamp;

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

    pub(crate) fn create_lsp_context_for_test(
    ) -> LspContext<IntoIter<TestSignalInput>, TestSignalBag> {
        LspContext::new(
            (0..1000)
                .map(|i| TestSignalInput {
                    value: i as u32,
                    timestamp: i,
                })
                .collect::<Vec<_>>()
                .into_iter(),
            true,
        )
    }

    pub(crate) fn create_lsp_context_for_test_from_input_slice(
        input: &[u32],
    ) -> LspContext<IntoIter<TestSignalInput>, TestSignalBag> {
        LspContext::new(
            input
                .iter()
                .zip(0..)
                .map(|(&v, t)| TestSignalInput {
                    value: v,
                    timestamp: t,
                })
                .collect::<Vec<_>>()
                .into_iter(),
            true,
        )
    }
}
