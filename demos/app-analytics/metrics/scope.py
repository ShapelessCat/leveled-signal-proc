from lsdl.const import Const
from lsdl.modules import SignalFilterBuilder, make_tuple
from lsdl.signal_processors.latch import EdgeTriggeredLatch
from schema import input

unconditional = SignalFilterBuilder(input.event_name).filter_values(
    "conviva_screen_view",
    "conviva_periodic_heartbeat",
    "conviva_application_foreground",
    "conviva_application_error",
    "conviva_page_view",
    "conviva_page_ping"
).build_clock_filter() # TODO: Try build_value_filter()

video_event_clock = SignalFilterBuilder(input.event_name).filter_values('conviva_video_events').build_clock_filter()
conditional = SignalFilterBuilder(input.conviva_video_events_name, video_event_clock).filter_values('c3.sdk.custom_event', 'c3.video.custom_event').build_clock_filter()

is_session_alive = EdgeTriggeredLatch(make_tuple(unconditional, conditional), data = Const(True), forget_duration = "90s")

session_id = is_session_alive.count_changes().add_metric("sessionId")

navigation_id = input.page_id.count_changes().add_metric("navId")

subscope = make_tuple(session_id, navigation_id).count_changes()
subscope.add_metric("subscope_id")
