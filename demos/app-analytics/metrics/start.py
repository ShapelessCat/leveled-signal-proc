from schema import input_signal
from scope import navigation_id, session_id

all_input_events_clock = input_signal.event_name.clock()

# Session start
# Original name and type in AA: `intvIsJustSessionStarted: Boolean`
previous_session_id = session_id.prior_value(all_input_events_clock)
(~(previous_session_id == session_id)).peek().add_metric("isSessionStartInterval")

# Navigation start
# Original name and type in AA: `intvIsJustPageSwitched: Boolean`
previous_navigation_id = navigation_id.prior_value(all_input_events_clock)
(~(previous_navigation_id == navigation_id)).peek().add_metric("isNavigationSwitchStartInterval")
