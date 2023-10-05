from lsdl.prelude import *
from schema import input

_unconditional = SignalFilterBuilder(input.event_name).filter_values(
    "conviva_screen_view",
    "conviva_periodic_heartbeat",
    "conviva_application_foreground",
    "conviva_application_error",
    "conviva_page_view",
    "conviva_page_ping"
).build_clock_filter() # TODO: Try build_value_filter()

_conditional = SignalFilterBuilder(input.event_name).filter_values('conviva_video_events')\
    .then_filter(input.conviva_video_events_name).filter_values('c3.sdk.custom_event', 'c3.video.custom_event')\
    .build_clock_filter()

_is_session_alive = make_tuple(_unconditional, _conditional).has_changed("90s")

session_id = _is_session_alive.count_changes().add_metric("sessionId")

_navigation_id = input.page_id.count_changes()

page_id = make_tuple(session_id, _navigation_id).count_changes()
