from enum import Enum

from const import CONVIVA_NETWORK_REQUEST
from lsdl.prelude import *
from schema import input
from scope import ScopeName, session_id, navigation_id

ResponseStatus = Enum('ResponseStatus', ['Success', 'Failure'])

network_request_duration = input.network_request_duration.parse("i32")

_network_request = SignalFilterBuilder(input.event_name).filter_values(CONVIVA_NETWORK_REQUEST)
_checked_network_request = _network_request.then_filter(network_request_duration > 0).then_filter(input.response_code)


def create_network_request_metrics_for(scope_singal, scope_name: ScopeName, status: ResponseStatus):
    global _checked_network_request
    op = '' if status is ResponseStatus.Success else '!'
    request_with_given_status = _checked_network_request.filter_fn('c', f"{op}c.starts_with('2')").build_clock_filter()
    count_with_given_status = request_with_given_status.count_changes()
    DiffSinceCurrentLevel(
        control=scope_singal,
        data=count_with_given_status
    ).add_metric(f"life{scope_name.name}{status.name}NetworkRequestCount")
    time_domain_fold(
        data=request_with_given_status,
        clock=count_with_given_status,
        scope=scope_singal
    ).add_metric(f"life{scope_name.name}{status.name}NetworkRequestDuration")


create_network_request_metrics_for(session_id, ScopeName.Session, ResponseStatus.Success)
create_network_request_metrics_for(navigation_id, ScopeName.Navigation, ResponseStatus.Success)

create_network_request_metrics_for(session_id, ScopeName.Session, ResponseStatus.Failure)
create_network_request_metrics_for(navigation_id, ScopeName.Navigation, ResponseStatus.Failure)
