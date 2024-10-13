from lsdl.processors import LivenessChecker
from schema import input_signal
from scope import ScopeName, navigation_id, session_id

is_user_active = LivenessChecker(
    liveness_clock=input_signal.event_name.clock(),
    ef_bind_var="_",
    ef_src="true",
)


def create_user_active_time_metric_for(scope_signal, scope_name: ScopeName):
    is_user_active.measure_duration_true().scope(scope_signal).add_metric(
        f"life_{scope_name.name.lower()}_user_active_time"
    )


create_user_active_time_metric_for(session_id, ScopeName.Session)
create_user_active_time_metric_for(navigation_id, ScopeName.Navigation)
