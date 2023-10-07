from lsdl.signal_processors.liveness import LivenessChecker
from scope import ScopeName, session_id, navigation_id
from schema import input

is_user_active = LivenessChecker(
    liveness_clock = input.event_name.clock(), 
    ef_bind_var = "e",
    ef_src = "true",
)

def create_user_active_time_metric_for(scope_signal, scope_name: ScopeName):
    global is_user_active
    is_user_active.measure_duration_true(scope_signal = scope_signal).add_metric(f"life{scope_name.name}UserActiveTime")

create_user_active_time_metric_for(session_id, ScopeName.Session)
# TODO: Fix this
create_user_active_time_metric_for(navigation_id, ScopeName.Navigation)