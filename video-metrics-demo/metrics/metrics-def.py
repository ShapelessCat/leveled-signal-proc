from lsdl.schema import InputSchemaBase, named, String, Integer, SessionizedInputSchemaBase
from lsdl import print_ir_to_stdout
from lsdl.signal import LeveledSignalBase
from lsdl.signal_processors import StateMachineBuilder, EdgeTriggeredLatch, SignalMapper

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
    # Sessionized signal descriptions
    bit_rate_default = -1

    def create_epoch_signal(self) -> LeveledSignalBase:
        return self.session_id.clock()

    def create_session_signal(self) -> LeveledSignalBase:
        return self.session_id.count_changes()


input = Input()

num_ps = input.player_state.map(bind_var = "s", lambda_src = f"""
    match s.as_str() {{
        "{PS_PLAYING}" => 0,
        "{PS_BUFFERING}" => 1,
        _ => 2,
    }}
""").annotate_type("i32")

# State
player_state = input.sessionized(num_ps, signal_clock = input.player_state.clock(), default_value = -1)
player_state.map(bind_var = "n", lambda_src = f"""
    match n {{
        0 => "{PS_PLAYING}",
        1 => "{PS_BUFFERING}",
        2 => "{PS_PAUSE}",
        _ => "",
    }} 
""").add_metric("playerState", typename="&'static str")
cdn = input.cdn.map(bind_var="s", lambda_src="s.to_string()").annotate_type("String")
input.sessionized(cdn, signal_clock = input.cdn.clock()).add_metric("cdn")
input.sessionized_bit_rate.add_metric("bitrate")

# Buffering time per session

is_buffering = (input.player_state == PS_BUFFERING)
is_buffering.measure_duration_true(scope_signal = input.session_signal).add_metric("bufferingTime")

has_been_playing = StateMachineBuilder(input.session_id.clock(), player_state)\
    .transition_fn("|&res: &bool, &state: &i32| res || state == 0")\
    .scoped(input.session_signal)\
    .build()

## Initial buffering time
is_init_buffering = (~has_been_playing & is_buffering)
is_init_buffering.measure_duration_true(scope_signal = input.session_signal).add_metric("initialBufferingTime")

## Re-buffering time
is_re_buffering = (has_been_playing & is_buffering)
is_re_buffering.measure_duration_true(scope_signal = input.session_signal).add_metric("rebufferingTime")

# ev - seek time
is_seek_start = (input.ev == EV_SEEK_START)
is_seek_start.measure_duration_true(scope_signal = input.session_signal).add_metric("seekTime")

# Debug
input.session_id.peek().add_metric("sessionId")

print_ir_to_stdout()