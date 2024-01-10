from schema import input_signal
from scope import navigation_id, session_id

all_input_events_clock = input_signal.event_name.clock()

# Session start
# Original name and type in AA: `intvIsJustSessionStarted: Boolean`
previous_session_id = session_id.prior_value(all_input_events_clock)
(~(previous_session_id == session_id)).add_metric("is_session_start_interval")

# Navigation start
# Original name and type in AA: `intvIsJustPageSwitched: Boolean`
previous_navigation_id = navigation_id.prior_value(all_input_events_clock)
(~(previous_navigation_id == navigation_id)) \
    .add_metric("is_navigation_switch_start_interval")
