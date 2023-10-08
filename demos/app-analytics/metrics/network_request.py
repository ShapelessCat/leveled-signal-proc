from enum import Enum
from lsdl.prelude import *

from const import CONVIVA_NETWORK_REQUEST
from schema import input
from scope import ScopeName, session_id, navigation_id

ResponseStatus = Enum('ResponseStatus', ['Success', 'Failure'])

network_request_duration = input.network_request_duration.parse("i32")

_network_request_filter_partial_builder =\
    SignalFilterBuilder(input.event_name)\
    .filter_values(CONVIVA_NETWORK_REQUEST)\
    .then_filter(network_request_duration > 0)
_request_succeeded = input.response_code.starts_with("2")


def create_network_request_metrics_for(scope_singal, scope_name: ScopeName, status: ResponseStatus):
    global _network_request_filter_partial_builder, _request_succeeded
    network_request_with_given_status_clock =\
        _network_request_filter_partial_builder\
        .then_filter(_request_succeeded if status == ResponseStatus.Success else (~_request_succeeded))\
        .build_clock_filter()
    count_with_given_status = network_request_with_given_status_clock.count_changes()
    DiffSinceCurrentLevel(
        control=scope_singal,
        data=count_with_given_status
    ).add_metric(f"life{scope_name.name}{status.name}NetworkRequestCount")
    time_domain_fold(
        data=network_request_duration,
        clock=network_request_with_given_status_clock,
        scope=scope_singal
    ).add_metric(f"life{scope_name.name}{status.name}NetworkRequestDuration")


create_network_request_metrics_for(session_id, ScopeName.Session, ResponseStatus.Success)
create_network_request_metrics_for(navigation_id, ScopeName.Navigation, ResponseStatus.Success)

create_network_request_metrics_for(session_id, ScopeName.Session, ResponseStatus.Failure)
create_network_request_metrics_for(navigation_id, ScopeName.Navigation, ResponseStatus.Failure)
