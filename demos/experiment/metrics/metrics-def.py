# extra-src: const.py schema.py scope.py first_video_attempt.py
from lsdl.prelude import measurement_config, print_ir_to_stdout
from schema import input_signal

import schema  # noqa: F401

import scope  # noqa: F401

import first_video_attempt  # noqa: F401

input_signal.peek_timestamp().add_metric("ts")

ts_plus1 = (
    input_signal
    .peek_timestamp()
    .map("x", "x + 1")
    .scope(scope.session_id)
    .add_metric("life_session_ts_plus1", typename="u64", need_interval_metric=True)
)

ts_plus1_plus2 = (
    input_signal
    .peek_timestamp()
    .map("x", "x + 1")
    .map("y", "y + 2")
    .add_metric("ts_plus1_plus2", "u64")
)

ts_plus1 \
    .combine('x', 'y', 'y - x', ts_plus1_plus2) \
    .scope(scope.session_id) \
    .add_metric("di", 'u64')


input_signal.encoded_fps.measure_linear_change().add_metric("encoded_frames")
input_signal \
    .inferred_rendered_fps \
    .measure_linear_change() \
    .add_metric("inferred_rendered_frames")

measurement_config() \
    .enable_measure_for_event() \
    .set_complementary_output_reset_switch("session_id")

print_ir_to_stdout()
