# extra-src: const.py schema.py scope.py first_video_attempt.py
from lsdl import measurement_config, print_ir_to_stdout
from schema import input_signal

import schema  # noqa: F401

import scope  # noqa: F401

import first_video_attempt  # noqa: F401

input_signal.peek_timestamp().add_metric("ts")

ts_plus1 = (
    input_signal
    .peek_timestamp()
    .scope(scope.session_id)
    .map("x", "x + 1")
    .add_metric("life_session_ts_plus1", typename="u64", need_interval_metric=True)
)

ts_plus1_plus2 = (
    input_signal
    .peek_timestamp()
    .scope(scope.session_id)
    .map("x", "x + 1")
    .map("y", "y + 2")
    .add_metric("ts_plus1_plus2", "u64")
)

ts_plus1 \
    .combine('x', 'y', 'y - x', ts_plus1_plus2) \
    .add_metric("diff", 'u64')

encoded_frames = input_signal \
    .encoded_fps \
    .measure_linear_change() \
    .scope(scope.session_id) \
    .add_metric("encoded_frames_count")

inferred_rendered_frames = input_signal \
    .inferred_rendered_fps \
    .measure_linear_change() \
    .scope(scope.session_id) \
    .add_metric("inferred_rendered_frames_count")

encoded_frames \
    .combine('x', 'y', 'x - y', inferred_rendered_frames) \
    .add_metric("dropped_frames_count", 'f64')

measurement_config() \
    .enable_measure_for_event() \
    .set_measure_at_measurement_true(scope.is_session_alive) \
    .set_complementary_output_reset_switch("session_id")

print_ir_to_stdout()
