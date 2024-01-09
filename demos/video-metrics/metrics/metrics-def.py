# extra-src: const.py schema.py
import const
from lsdl.prelude import print_ir_to_stdout, processing_config, StateMachineBuilder
from schema import input_signal


# Sessionized states
input_signal.sessionized_player_state.add_metric("player_state")
input_signal.sessionized_cdn.add_metric("cdn")
input_signal.sessionized_bit_rate.add_metric("bitrate")

# Total buffering time per session
is_buffering = input_signal.sessionized_player_state == const.PS_BUFFERING
is_buffering\
    .measure_duration_true()\
    .scope(input_signal.session_signal)\
    .add_metric("buffering_time")

# - Initial buffering time
has_been_playing = (
    StateMachineBuilder(
        input_signal.session_id.clock(),
        input_signal.player_state
    )
    .transition_fn(f"|&res: &bool, state: &String| res || state == \"{const.PS_PLAYING}\"")
    .scoped(input_signal.session_signal)
    .build()
)

(~has_been_playing & is_buffering)\
    .measure_duration_true()\
    .scope(input_signal.session_signal)\
    .add_metric("initial_buffering_time")

# - Re-buffering time
(has_been_playing & is_buffering)\
    .measure_duration_true()\
    .scope(input_signal.session_signal)\
    .add_metric("rebuffering_time")

# ev - seek time
(input_signal.ev == const.EV_SEEK_START)\
    .measure_duration_true()\
    .scope(input_signal.session_signal)\
    .add_metric("seek_time")

processing_config().set_merge_simultaneous_moments(should_merge=False)

# Dump IR from metric definitions
print_ir_to_stdout()
