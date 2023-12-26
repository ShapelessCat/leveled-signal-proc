from lsdl.prelude import LivenessChecker
from scope import ScopeName, session_id, navigation_id
from schema import input_signal


is_user_active = LivenessChecker(
    liveness_clock=input_signal.event_name.clock(),
    ef_bind_var="_",
    ef_src="true",
)


def create_user_active_time_metric_for(scope_signal, scope_name: ScopeName):
    is_user_active\
        .measure_duration_true(scope_signal=scope_signal)\
        .add_metric(f"life{scope_name.name}UserActiveTime")


create_user_active_time_metric_for(session_id, ScopeName.Session)
create_user_active_time_metric_for(navigation_id, ScopeName.Navigation)
