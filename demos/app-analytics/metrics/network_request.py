from enum import Enum

from lsdl.processors import SignalFilterBuilder, time_domain_fold

import const
from schema import input_signal
from scope import ScopeName, navigation_id, session_id

ResponseStatus = Enum("ResponseStatus", ["Success", "Failure"])

network_request_duration = input_signal.network_request_duration.parse("i32")

_network_request_filter_partial_builder = (
    SignalFilterBuilder(input_signal.event_name)
    .filter_values(const.CONVIVA_NETWORK_REQUEST)
    .then_filter(network_request_duration > 0)
)
_request_succeeded = input_signal.response_code.starts_with("2")


def create_network_request_metrics_for(
    scope_signal, scope_name: ScopeName, status: ResponseStatus
):
    global _network_request_filter_partial_builder, _request_succeeded
    is_success = status == ResponseStatus.Success
    network_request_with_given_status_clock = (
        _network_request_filter_partial_builder.then_filter(
            _request_succeeded if is_success else (~_request_succeeded)
        ).build_clock_filter()
    )
    count_with_given_status = network_request_with_given_status_clock.count_changes()
    scope_and_status = f"{scope_name.name.lower()}_{status.name.lower()}"
    count_with_given_status.peek().scope(scope_signal).add_metric(
        f"life_{scope_and_status}_network_request_count"
    )
    time_domain_fold(
        data=network_request_duration,
        clock=network_request_with_given_status_clock,
        scope=scope_signal,
    ).add_metric(f"life_{scope_and_status}_network_request_duration")


create_network_request_metrics_for(
    session_id, ScopeName.Session, ResponseStatus.Success
)
create_network_request_metrics_for(
    navigation_id, ScopeName.Navigation, ResponseStatus.Success
)

create_network_request_metrics_for(
    session_id, ScopeName.Session, ResponseStatus.Failure
)
create_network_request_metrics_for(
    navigation_id, ScopeName.Navigation, ResponseStatus.Failure
)
