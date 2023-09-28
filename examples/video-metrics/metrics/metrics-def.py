from lsdl.schema import named, String, Integer, SessionizedInputSchemaBase
from lsdl import print_ir_to_stdout
from lsdl.signal import LeveledSignalBase
from lsdl.signal_processors import StateMachineBuilder

PS_PLAYING = "playing"
PS_BUFFERING = "buffering"
PS_PAUSE = "pause"

EV_SEEK_START = "seek start"
EV_SEEK_END = "seek end"

class Input(SessionizedInputSchemaBase):
    _timestamp_key = "timestamp"
    session_id     = named("sessionId",   String())
    player_state   = named("PlayerState", String())
    cdn            = named("CDN",         String())
    bit_rate       = named("BitRate",     Integer())
    ev             = named("ev",          String())
    
    bit_rate_default = -1
    def create_epoch_signal(self) -> LeveledSignalBase:
        return self.session_id.clock()
    def create_session_signal(self) -> LeveledSignalBase:
        return self.session_id.count_changes()

input = Input()

# Sessionized states
input.sessionized_player_state.add_metric("playerState")
input.sessionized_cdn.add_metric("cdn")
input.sessionized_bit_rate.add_metric("bitrate")

# Total buffering time per session
is_buffering = (input.sessionized_player_state == PS_BUFFERING)
is_buffering.measure_duration_true(scope_signal = input.session_signal).add_metric("bufferingTime")

## Initial buffering time
has_been_playing = StateMachineBuilder(input.session_id.clock(), input.player_state)\
    .transition_fn(f"|&res: &bool, state: &String| res || state == \"{PS_PLAYING}\"")\
    .scoped(input.session_signal)\
    .build()

is_init_buffering = (~has_been_playing & is_buffering)
is_init_buffering.measure_duration_true(scope_signal = input.session_signal).add_metric("initialBufferingTime")

## Re-buffering time
is_re_buffering = (has_been_playing & is_buffering)
is_re_buffering.measure_duration_true(scope_signal = input.session_signal).add_metric("rebufferingTime")

# ev - seek time
is_seek_start = (input.ev == EV_SEEK_START)
is_seek_start.measure_duration_true(scope_signal = input.session_signal).add_metric("seekTime")

print_ir_to_stdout()