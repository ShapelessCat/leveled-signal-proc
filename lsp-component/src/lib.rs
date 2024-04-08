pub mod measurements;
pub mod processors;

#[cfg(test)]
pub(crate) mod test {
    use std::vec::IntoIter;

    use lsp_runtime::context::{InputSignalBag, LspContext, WithTimestamp};
    use lsp_runtime::Timestamp;

    #[derive(Clone, Default)]
    pub(crate) struct TestSignalBag<T> {
        pub(crate) value: T,
    }

    #[derive(Clone, Default)]
    pub(crate) struct TestSignalInput<T> {
        pub(crate) value: T,
        pub(crate) timestamp: Timestamp,
    }

    impl<T> WithTimestamp for TestSignalInput<T> {
        fn timestamp(&self) -> u64 {
            self.timestamp
        }
    }

    impl<T: Clone + Default> InputSignalBag for TestSignalBag<T> {
        type Input = TestSignalInput<T>;

        fn patch(&mut self, patch: Self::Input) {
            self.value = patch.value;
        }
    }

    pub(crate) fn create_lsp_context_for_test(
    ) -> LspContext<IntoIter<TestSignalInput<u32>>, TestSignalBag<u32>> {
        let data = 0..100;
        let input = &data.collect::<Vec<u32>>()[..];
        create_lsp_context_for_test_from_input(input)
    }

    pub(crate) fn create_lsp_context_for_test_from_input<T: Clone + Default>(
        input: &[T],
    ) -> LspContext<IntoIter<TestSignalInput<T>>, TestSignalBag<T>> {
        LspContext::new(
            input
                .iter()
                .zip(0..)
                .map(|(v, t)| TestSignalInput {
                    value: v.to_owned(),
                    timestamp: t,
                })
                .collect::<Vec<_>>()
                .into_iter(),
            true,
        )
    }
}
