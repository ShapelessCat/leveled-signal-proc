from lsdl.schema import InputSchemaBase, named, String, Integer
from lsdl import print_ir_to_stdout
from lsdl.signal_processors import StateMachineBuilder, EdgeTriggeredLatch, SignalMapper

PS_PLAYING = "playing"
PS_BUFFERING = "buffering"
PS_PAUSE = "pause"

EV_SEEK_S = "seek start"
EV_SEEK_E = "seek end"

class Input(InputSchemaBase):
    _timestamp_key = "timestamp"
    session_id     = named("sessionId",   String())
    player_state   = named("PlayerState", String())
    cdn            = named("CDN",         String())
    bit_rate       = named("BitRate",     Integer())
    ev             = named("ev",          String())

input = Input()

# Buffering time per session

num_sid = input.session_id.count_changes()
is_buffering = (input.player_state == PS_BUFFERING)
is_buffering.measure_duration_true(scope_signal = num_sid).add_metric("bufferingTime")

## Re-buffering time
num_ps = input.player_state.map(bind_var = "s", lambda_src = f"""
    match s.as_str() {{
        "{PS_PLAYING}" => 0,
        "{PS_BUFFERING}" => 1,
        _ => 2,
    }}
""")

ps_event_ts = EdgeTriggeredLatch(input.player_state.clock(), input.session_id.clock())
session_ts = EdgeTriggeredLatch(num_sid, input.session_id.clock())
sessionized_ps = SignalMapper(
    bind_var = "(session_ts, ets, ps)", 
    lambda_src = "if *ets < *session_ts { -1 } else {*ps}", 
    upstream = [session_ts, ps_event_ts, num_ps])
sessionized_ps.add_metric("playerState", "i32")

has_been_playing = StateMachineBuilder(input.session_id.clock(), sessionized_ps)\
    .transition_fn("|&res: &bool, &state: &i32| res || state == 0")\
    .scoped(num_sid)\
    .build()

is_init_buffering = (has_been_playing & is_buffering)
is_init_buffering.measure_duration_true(scope_signal = num_sid).add_metric("RebufferingTime")

input.session_id.peek().add_metric("sessionId")

print_ir_to_stdout()
