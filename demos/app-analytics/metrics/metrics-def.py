# extra-src: app_startup.py const.py first_video_attempt.py network_request.py page_load.py schema.py scope.py user_active_time.py
from lsdl.prelude import *

import schema

import scope

# App Startup
import app_startup

# Network Request
import network_request

# Page Load
import page_load

# Session Start Status
# Session End Status

# User Active Time
import user_active_time

# First Video Attempt
import first_video_attempt

# TODO: Measurement Policy for 1 min
measurement_config().enable_measure_for_event()

print_ir_to_stdout()
