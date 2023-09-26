from lsdl.schema import InputSchemaBase, named, String, Integer
from lsdl import print_ir_to_stdout
from lsdl.signal_processors import StateMachineBuilder

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

has_been_playing = StateMachineBuilder(input.session_id.clock(), num_ps)\
    .transition_fn("|s: &bool, ps: &i32| *s || *ps == 0")\
    .scoped(num_sid)\
    .build()

is_init_buffering = (has_been_playing & is_buffering)
is_init_buffering.measure_duration_true(scope_signal = num_sid).add_metric("RebufferingTime")

input.session_id.peek().add_metric("sessionId")

print_ir_to_stdout()
