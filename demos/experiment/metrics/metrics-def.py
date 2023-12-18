# extra-src: const.py schema.py scope.py
from lsdl.prelude import *
from schema import input_signal

import schema

input_signal.peek_timestamp().add_metric("ts")

input_signal.encoded_fps.measure_linear_change().add_metric("encoded_frames")
input_signal.inferred_rendered_fps.measure_linear_change().add_metric("inferred_rendered_frames")

measurement_config() \
    .enable_measure_for_event() \
    .set_limit_side_signal(signal=input_signal.map('x', 'true'))

print_ir_to_stdout()
