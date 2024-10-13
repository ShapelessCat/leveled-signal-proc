# extra-src: app_startup.py const.py first_video_attempt.py network_request.py
# extra-src: page_load.py schema.py scope.py start.py user_active_time.py

# App Startup
import app_startup  # noqa: F401

# First Video Attempt
import first_video_attempt  # noqa: F401

# Network Request
import network_request  # noqa: F401

# Page Load
import page_load  # noqa: F401
import schema  # noqa: F401
import scope  # noqa: F401

# Start Status
import start  # noqa: F401

# User Active Time
import user_active_time  # noqa: F401
from lsdl import measurement_config, print_ir_to_stdout

# End Status


# TODO: Measurement Policy for 1 min
measurement_config().enable_measure_for_event()

print_ir_to_stdout()
