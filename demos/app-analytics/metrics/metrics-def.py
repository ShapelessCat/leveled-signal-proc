# extra-src: app_startup.py schema.py scope.py user_active_time.py
from lsdl import print_ir_to_stdout, measurement_config
import schema

# App Startup Time
import app_startup
# Network Request Count
# Network Request Duration
# Page Load
# Session Start Status
# Session End Status
# User Active Time
import user_active_time
# First Video Attempt

#TODO: Measurement Policy
measurement_config().enable_measure_for_event()
print_ir_to_stdout()