# extra-src: const.py schema.py scope.py first_video_attempt.py
from lsdl.prelude import *
from schema import input_signal

import schema

import first_video_attempt

import scope

input_signal.peek_timestamp().add_metric("ts")

ts_plus1 = input_signal.peek_timestamp().map("x", "x + 1").add_metric("ts_plus1", "u64")
ts_plus1_plus2 = input_signal.peek_timestamp().map("x", "x + 1").map("y", "y + 2").add_metric("ts_plus1_plus2", "u64")
ts_plus1.combine('x', 'y', 'y - x', ts_plus1_plus2).add_metric("diff", 'u64')


input_signal.encoded_fps.measure_linear_change().add_metric("encoded_frames")
input_signal.inferred_rendered_fps.measure_linear_change().add_metric("inferred_rendered_frames")

measurement_config().enable_measure_for_event()

print_ir_to_stdout()
