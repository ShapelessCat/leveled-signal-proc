import const
from lsdl.prelude import Const, SignalFilterBuilder, StateMachineBuilder
from schema import input_signal
from scope import session_id


video_attempt_clock =\
    SignalFilterBuilder(input_signal.conviva_video_events_name)\
        .filter_values(const.VIDEO_ATTEMPT)\
        .build_clock_filter()

video_attempt =\
    StateMachineBuilder(video_attempt_clock, Const(1))\
        .transition_fn("|&s: &i32, _: &i32| (s+1).min(2)")\
        .scoped(session_id)\
        .build()

video_attempt.add_metric("video_attempt", 'i32')

has_first_video_attempt = video_attempt >= 1
has_first_video_attempt.peek().add_metric('lifeSessionHasFirstVideoAttempted')

before_first_video_attempt = ~(video_attempt == 1).has_been_true()
before_first_video_attempt.add_metric('has_been_true')
before_first_video_attempt\
    .measure_duration_true(session_id)\
    .add_metric('lifeSessionDurationBeforeFirstVideoAttempt')

is_first_video_attempt = video_attempt == 1
is_first_video_attempt.peek().add_metric('intervalHasFirstVideoAttempted')

# TODO:
# 1. Introduce measure combinator, and add the functionality to config one-sided measurement.
# 2. Introduce post-processing for interval metrics
# 'intervalDurationBeforeFirstVideoAttempt'