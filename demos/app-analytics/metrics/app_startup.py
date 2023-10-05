
from lsdl.const import Const
from lsdl.measurements.diff import DiffSinceCurrentLevel
from lsdl.modules import SignalFilterBuilder, time_domain_fold
from lsdl.signal import If
from lsdl.signal_processors.accumulator import Accumulator
from schema import input
from scope import session_id

screen_view_clock = SignalFilterBuilder(input.event_name).filter_values('conviva_screen_view').build_clock_filter()
start = input.app_startup_start.parse("i32")
end = input.app_startup_end.parse("i32")
duration = end - start
is_valid_app_startup_duration = (input.app_startup_previous_exist == "") & (start > 0) & (duration > 0) & (duration < 300_000)
app_startup_time = If(is_valid_app_startup_duration, duration, Const(-1))

# App Startup Count
total_startup_count = Accumulator(app_startup_time, Const(1), filter_lambda = "|&t| t > 0")
total_startup_count.add_metric("totalCount")
DiffSinceCurrentLevel(control = session_id, data = total_startup_count).add_metric("lifeSessionStartUpCount")

time_domain_fold(app_startup_time, init_state = 0, fold_method = "max", scope = session_id).add_metric("lifeSessionMaxStartupDuration")
time_domain_fold(app_startup_time, fold_method = "sum", scope = session_id).add_metric("lifeSessionStartUpDuration")
