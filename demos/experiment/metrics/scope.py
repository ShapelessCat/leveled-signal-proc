from enum import Enum

import const
from lsdl.processors import SignalFilterBuilder, make_tuple
from schema import input_signal

_unconditional = (
    SignalFilterBuilder(input_signal.event_name)
    .filter_values(*const.UNCONDITIONAL_CRITICAL_EVENT_NAMES)
    .build_clock_filter()
)

_conditional = (
    SignalFilterBuilder(input_signal.event_name)
    .filter_values(const.CONVIVA_VIDEO_EVENTS)
    .then_filter(input_signal.conviva_video_events_name)
    .filter_values(*const.CRITICAL_CONVIVA_VIDEO_EVENTS_NAMES)
    .build_clock_filter()
)

is_session_alive = make_tuple(_unconditional, _conditional).has_changed("90s")

is_session_alive.add_metric("is_session_alive")

session_id = is_session_alive.count_changes().add_metric("session_id")

_page_id = input_signal.page_id.count_changes()
_screen_id = input_signal.screen_id.count_changes()

navigation_id = (
    make_tuple(session_id, _page_id, _screen_id)
    .count_changes()
    .add_metric("navigation_id")
)

ScopeName = Enum("ScopeName", ["Session", "Navigation"])
