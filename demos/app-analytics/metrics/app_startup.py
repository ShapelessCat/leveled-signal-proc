from const import CONVIVA_SCREEN_VIEW
from lsdl.prelude import *
from schema import input
from scope import session_id, navigation_id

start = input.app_startup_start.parse("i32")
end = input.app_startup_end.parse("i32")
duration = end - start
is_valid_app_startup_duration = (input.app_startup_previous_exist == "") & (start > 0) & (duration > 0) & (duration < 300_000)
app_startup_time = If(is_valid_app_startup_duration, duration, Const(-1))

app_startup_clock =\
    SignalFilterBuilder(input.event_name)\
    .filter_values(CONVIVA_SCREEN_VIEW)\
    .then_filter(app_startup_time > 0)\
    .build_clock_filter()

total_startup_count = app_startup_clock.count_changes()

def fold_app_startup_time(method, init = None, scope = session_id):
    global app_startup_time, app_startup_clock, session_id
    return time_domain_fold(
        data = app_startup_time, 
        clock = app_startup_clock, 
        init_state = init, 
        fold_method = method, 
        scope = scope)

def create_app_startup_metrics_for(scope_signal, scope_name):
    global total_startup_count
    DiffSinceCurrentLevel(control = scope_signal, data = total_startup_count).add_metric(f"life{scope_name}StartUpCount")
    fold_app_startup_time("max", init = 0, scope = scope_signal).add_metric(f"life{scope_name}MaxStartupDuration")
    fold_app_startup_time("sum", scope = scope_signal).add_metric(f"life{scope_name}StartUpDuration")

create_app_startup_metrics_for(session_id, "Session")
create_app_startup_metrics_for(navigation_id, "Page")