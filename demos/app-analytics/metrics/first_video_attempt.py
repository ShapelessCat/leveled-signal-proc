import const
from lsdl.prelude import Const, SignalFilterBuilder, StateMachineBuilder
from schema import input_signal
from scope import session_id


video_attempt_clock =\
    SignalFilterBuilder(input_signal.conviva_video_events_name)\
        .filter_values(const.VIDEO_ATTEMPT)\
        .build_clock_filter()


has_first_video_attempt =\
    StateMachineBuilder(video_attempt_clock, Const(1))\
        .transition_fn("|&s: &i32, _: &i32| (s+1).min(2)")\
        .scoped(session_id)\
        .build() >= 1

has_first_video_attempt.peek().add_metric('lifeSessionHasFirstVideoAttempted')

(~has_first_video_attempt)\
    .measure_duration_true(scope_signal=session_id)\
    .add_metric("lifeSessionDurationBeforeFirstVideoAttempt")
