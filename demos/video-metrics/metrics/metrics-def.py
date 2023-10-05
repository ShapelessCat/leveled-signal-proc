# extra-src: const.py schema.py
from lsdl import print_ir_to_stdout
from lsdl.signal_processors import StateMachineBuilder
import const
import schema

# Create the instant for input signals
input = schema.Input()

# Sessionized states
input.sessionized_player_state.add_metric("playerState")
input.sessionized_cdn.add_metric("cdn")
input.sessionized_bit_rate.add_metric("bitrate")

# Total buffering time per session
is_buffering = (input.sessionized_player_state == const.PS_BUFFERING)
is_buffering.measure_duration_true(scope_signal = input.session_signal).add_metric("bufferingTime")

## Initial buffering time
has_been_playing = StateMachineBuilder(input.session_id.clock(), input.player_state)\
    .transition_fn(f"|&res: &bool, state: &String| res || state == \"{const.PS_PLAYING}\"")\
    .scoped(input.session_signal)\
    .build()

(~has_been_playing & is_buffering).measure_duration_true(scope_signal = input.session_signal).add_metric("initialBufferingTime")

## Re-buffering time
(has_been_playing & is_buffering).measure_duration_true(scope_signal = input.session_signal).add_metric("rebufferingTime")

# ev - seek time
(input.ev == const.EV_SEEK_START).measure_duration_true(scope_signal = input.session_signal).add_metric("seekTime")

# Dump IR from metric defnitions
print_ir_to_stdout()