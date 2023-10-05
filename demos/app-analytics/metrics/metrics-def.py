# extra-src: schema.py
from lsdl.measurements.diff import DiffSinceCurrentLevel
from schema import Input
from lsdl.signal_processors import EdgeTriggeredLatch, StateMachineBuilder, Accumulator, LivenessChecker
from lsdl import print_ir_to_stdout, measurement_config
from lsdl.const import Const
from lsdl.modules import SignalFilterBuilder, make_tuple
from lsdl.signal import If
from lsdl.signal_processors.signal_gen import SquareWave

input = Input()

unconditional = SignalFilterBuilder(input.event_name).filter_values(
    "conviva_screen_view",
    "conviva_periodic_heartbeat",
    "conviva_application_foreground",
    "conviva_application_error",
    "conviva_page_view",
    "conviva_page_ping"
).build_clock_filter() # TODO: Try build_value_filter()

b = SignalFilterBuilder(input.event_name).filter_values('conviva_video_events').build_clock_filter()
conditional = SignalFilterBuilder(input.conviva_video_events_name, b).filter_values('c3.sdk.custom_event', 'c3.video.custom_event').build_clock_filter()

keep_session_alive_clock = make_tuple(unconditional, conditional).add_metric('ac', '(u64, u64)')
# input.event_name.peek().add_metric('ename')

is_session_alive = EdgeTriggeredLatch(keep_session_alive_clock, data = Const(True), forget_duration = 90000000000)

session_id = is_session_alive.count_changes().add_metric("sessionId")

navigation_id = input.page_id.count_changes().add_metric("navId")

subscope = make_tuple(session_id, navigation_id).count_changes()
subscope.add_metric("subscope_id")

## TODO:
# measurement_config()\
#     .disable_measure_for_event()\
#     .set_trigger_signal(make_tuple(SquareWave("60s"), is_session_alive))


# App Startup Duration
screen_view_only = SignalFilterBuilder(input.event_name).filter_values('conviva_screen_view').build_clock_filter()
numeric_app_startup_start = input.app_startup_start.map(bind_var = "s", lambda_src = "s.parse::<i32>().unwrap_or(-1)")
numeric_app_startup_end = input.app_startup_end.map(bind_var = "s", lambda_src = "s.parse::<i32>().unwrap_or(-1)")
app_startup_time = If(
    (input.app_startup_previous_exist == "") \
        & (numeric_app_startup_start > 0) \
        & (numeric_app_startup_end > numeric_app_startup_start) \
        & (numeric_app_startup_end - numeric_app_startup_start < 300000), 
    (numeric_app_startup_end - numeric_app_startup_start).annotate_type("i32"),
    Const(-1)
)

app_startup_time.peek().add_metric("startupTime", "i32")

# App Startup Count
Accumulator(app_startup_time, Const(1), filter_lambda = "|&t| t> 0").add_metric("StartUpCount")

StateMachineBuilder(clock = app_startup_time, data = app_startup_time).transition_fn(fn = """
|&max: &i32, &current : &i32| current.max(max)
""").scoped(session_id).build().add_metric("lifeSessionMaxStartupDuration", "i32")

StateMachineBuilder(clock = app_startup_time, data = app_startup_time).transition_fn(fn = """
|&sum: &i32, &current : &i32| sum + current.max(0)
""").scoped(session_id).build().add_metric("lifeSessionStartupDuration", "i32")

# Network Request Count
# Network Request Duration
# Page Load
# Session Start Status
# Session End Status
# User Active Time
is_user_active = LivenessChecker(
    liveness_clock = input.event_name.clock(), 
    ef_bind_var = "e",
    ef_src = "true",
).measure_duration_true(scope_signal = session_id).add_metric("lifeSessionUserActiveTime")
# First Video Attempt

print_ir_to_stdout()