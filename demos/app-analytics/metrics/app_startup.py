
from lsdl.const import Const
from lsdl.measurements.diff import DiffSinceCurrentLevel
from lsdl.modules import SignalFilterBuilder, time_domain_fold
from lsdl.signal import If
from lsdl.signal_processors.accumulator import Accumulator
from schema import input
from scope import session_id


start = input.app_startup_start.parse("i32")
end = input.app_startup_end.parse("i32")
duration = end - start
is_valid_app_startup_duration = (input.app_startup_previous_exist == "") & (start > 0) & (duration > 0) & (duration < 300_000)
app_startup_time = If(is_valid_app_startup_duration, duration, Const(-1))

screen_view_clock = SignalFilterBuilder(input.event_name).filter_values('conviva_screen_view').build_clock_filter()
app_startup_clock = SignalFilterBuilder(app_startup_time > 0, screen_view_clock).filter_true().build_clock_filter()

# App Startup Count
total_startup_count = app_startup_clock.count_changes()
DiffSinceCurrentLevel(control = session_id, data = total_startup_count).add_metric("lifeSessionStartUpCount")

def fold_app_startup_time(method, init = None):
    global app_startup_time, app_startup_clock, session_id
    return time_domain_fold(
        data = app_startup_time, 
        clock = app_startup_clock, 
        init_state = init, 
        fold_method = method, 
        scope = session_id)

fold_app_startup_time("max", 0).add_metric("lifeSessionMaxStartupDuration")
fold_app_startup_time("sum").add_metric("lifeSessionStartUpDuration")