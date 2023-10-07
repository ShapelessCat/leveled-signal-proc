# extra-src: app_startup.py const.py network_request.py schema.py scope.py user_active_time.py
from lsdl import print_ir_to_stdout, measurement_config
import schema

# App Startup
import app_startup

# Network Request
import network_request

# Page Load
# Session Start Status
# Session End Status

# User Active Time
import user_active_time

# First Video Attempt

#TODO: Measurement Policy
measurement_config().enable_measure_for_event()

print_ir_to_stdout()