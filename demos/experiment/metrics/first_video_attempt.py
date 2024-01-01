import const
from lsdl.prelude import Const, SignalFilterBuilder, StateMachineBuilder
from schema import input_signal
from scope import session_id


video_attempt_clock = (
    SignalFilterBuilder(input_signal.conviva_video_events_name)
    .filter_values(const.VIDEO_ATTEMPT)
    .build_clock_filter()
)
video_attempt = (
    StateMachineBuilder(video_attempt_clock, Const(1))
    .transition_fn("|&s: &i32, _: &i32| (s+1).min(2)")
    .scoped(session_id)
    .build()
)
duration_before_first_video_attempt = ~(video_attempt == 1).has_been_true()
# duration_before_first_video_attempt\
#     .measure_duration_true(session_id)\
#     .add_metric('life_session_duration_before_first_video_attempt')

duration_before_first_video_attempt\
    .measure_duration_true()\
    .scope(session_id)\
    .add_metric('life_session_duration_before_first_video_attempt')

duration_before_first_video_attempt\
    .measure_duration_true()\
    .scope(session_id)\
    .map("x", "x + 1")\
    .add_metric('plus1_life_session_duration_before_first_video_attempt', 'u64')
