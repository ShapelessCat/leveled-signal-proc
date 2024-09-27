"""Constants for building metrics."""

PLATFORM_MOBILE = "mob"
PLATFORM_WEB = "web"

# Critical events:
# Events that can keep an application session alive in the next 90 seconds.

# - Values of 'event_name' (unconditional):
CONVIVA_SCREEN_VIEW = "conviva_screen_view"
CONVIVA_PERIODIC_HEARTBEAT = "conviva_periodic_heartbeat"
CONVIVA_APPLICATION_FOREGROUND = "conviva_application_foreground"
CONVIVA_APPLICATION_ERROR = "conviva_application_error"
CONVIVA_PAGE_VIEW = "conviva_page_view"
CONVIVA_PAGE_PING = "conviva_page_ping"

UNCONDITIONAL_CRITICAL_EVENT_NAMES = [
    CONVIVA_SCREEN_VIEW,
    CONVIVA_PERIODIC_HEARTBEAT,
    CONVIVA_APPLICATION_FOREGROUND,
    CONVIVA_APPLICATION_ERROR,
    CONVIVA_PAGE_VIEW,
    CONVIVA_PAGE_PING,
]

# - Value of 'event_name' (conditional):
CONVIVA_VIDEO_EVENTS = "conviva_video_events"
#   + Condition: critical values of 'conviva_video_events_name':
CRITICAL_CONVIVA_VIDEO_EVENTS_NAMES = ["c3.sdk.custom_event", "c3.video.custom_event"]
# 'conviva_video_events_name', not critical
VIDEO_ATTEMPT = "c3.video.attempt"

# Other events:
CONVIVA_NETWORK_REQUEST = "conviva_network_request"
# - Network request related keys:
RESPONSE_CODE = "response_code"
NETWORK_REQUEST_DURATION = "network_request_duration"

# Thresholds:
PAGE_LOAD_TIME_THRESHOLD = 90000
SCREEN_LOADTIME_THRESHOLD = 60000
