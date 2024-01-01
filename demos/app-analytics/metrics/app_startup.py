import const
from lsdl.prelude import Const, If, SignalFilterBuilder, time_domain_fold
from schema import input_signal
from scope import ScopeName, session_id, navigation_id

start = input_signal.app_startup_start.parse("i32")
end = input_signal.app_startup_end.parse("i32")
duration = end - start
is_valid_app_startup_duration = (
    (input_signal.app_startup_previous_exist == "") &
    (start > 0) &
    (0 < duration < 300_000)
)
app_startup_time = If(is_valid_app_startup_duration, duration, Const(-1))

app_startup_clock = (
    SignalFilterBuilder(input_signal.event_name)
    .filter_values(const.CONVIVA_SCREEN_VIEW)
    .then_filter(app_startup_time > 0)
    .build_clock_filter()
)

total_startup_count = app_startup_clock.count_changes()


def fold_app_startup_time(scope, method, init=None):
    global app_startup_time, app_startup_clock
    return time_domain_fold(
        data=app_startup_time,
        clock=app_startup_clock,
        init_state=init,
        fold_method=method,
        scope=scope)


def create_app_startup_metrics_for(scope_signal, scope_name: ScopeName):
    global total_startup_count
    total_startup_count\
        .peek()\
        .scope(scope_signal)\
        .add_metric(f"life{scope_name.name}StartupCount")

    fold_app_startup_time(
        scope_signal,
        "max",
        init=0
    ).add_metric(f"life{scope_name.name}MaxStartupDuration")

    fold_app_startup_time(
        scope_signal, "sum"
    ).add_metric(f"life{scope_name.name}StartupDuration")


create_app_startup_metrics_for(session_id, ScopeName.Session)
create_app_startup_metrics_for(navigation_id, ScopeName.Navigation)
