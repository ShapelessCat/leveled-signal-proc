from const import CONVIVA_VIDEO_EVENTS, CRITICAL_CONVIVA_VIDEO_EVENTS_NAMES, UNCONDITIONAL_CRITICAL_EVENT_NAMES
from lsdl.prelude import *
from schema import input

_unconditional =\
    SignalFilterBuilder(input.event_name)\
    .filter_values(*UNCONDITIONAL_CRITICAL_EVENT_NAMES)\
    .build_clock_filter() # TODO: Try build_value_filter()

_conditional =\
    SignalFilterBuilder(input.event_name)\
    .filter_values(CONVIVA_VIDEO_EVENTS)\
    .then_filter(input.conviva_video_events_name)\
    .filter_values(*CRITICAL_CONVIVA_VIDEO_EVENTS_NAMES)\
    .build_clock_filter()

_is_session_alive = make_tuple(_unconditional, _conditional).has_changed("90s")

session_id = _is_session_alive.count_changes().add_metric("sessionId")

_navigation_id = input.page_id.count_changes()

page_id = make_tuple(session_id, _navigation_id).count_changes()
