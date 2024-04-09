.DEFAULT_GOAL := regression

.DELETE_ON_ERROR:

.PHONY: regression run-regression clean-checkpoints

regression: clean-checkpoints run-regression clean-checkpoints

clean-checkpoints:
	rm -f ./demos/*/*checkpoint*.json

run-regression:
	cargo clean

	cargo build

	cargo test

#	Make sure demos work and work well (TODO: verify results)
	# Run demos and only show the counts of the each demo output rather than output details.
	./target/debug/app-analytics ./assets/data/app-analytics-metrics-demo-input.jsonl | wc -l
	./target/debug/experiment ./assets/data/experiment-demo-input.jsonl | wc -l
	./target/debug/video-metrics ./assets/data/video-metrics-demo-input.jsonl | wc -l

#	TODO: Missing tests
#         1. lsp-runtime/examples (run and verify results),
#         2. lsp-codegen-test (need fix, run, and verify results),
#         3. lsdl/examples (run and verify results)
