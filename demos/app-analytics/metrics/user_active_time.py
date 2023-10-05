from lsdl.signal_processors.liveness import LivenessChecker
from scope import session_id
from schema import input

is_user_active = LivenessChecker(
    liveness_clock = input.event_name.clock(), 
    ef_bind_var = "e",
    ef_src = "true",
).measure_duration_true(scope_signal = session_id).add_metric("lifeSessionUserActiveTime")