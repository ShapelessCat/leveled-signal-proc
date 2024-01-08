import logging
from typing import Any, Self

from .measurement import MeasurementBase
from .rust_code import COMPILER_INFERABLE_TYPE, RustCode
from .signal import SignalBase

logging.basicConfig(encoding='utf-8', level=logging.INFO)


class _ProcessingConfiguration:
    """The configuration for processing policy.
    """
    def __init__(self):
        self._merge_simultaneous_moments = True

    def set_merge_simultaneous_moments(self, should_merge: bool) -> Self:
        """Set the rule for handling simultaneous moments."""
        self._merge_simultaneous_moments = should_merge
        return self

    def to_dict(self) -> dict[str, Any]:
        """Dump the processing policy into a dictionary that can be JSONified."""
        return {"merge_simultaneous_moments": self._merge_simultaneous_moments}


def _make_processing_configuration():
    config = _ProcessingConfiguration()
    return lambda: config


processing_config = _make_processing_configuration()


class _MeasurementConfiguration:
    """The configuration for measurement policy.

    In LSP, there are two method to trigger a measurement:
    1. triggered by an input event;
    2. triggered by a signal edge.
    """
    def __init__(self):
        self._measure_at_event_lambda = "|_| true"
        self._measure_on_edge = None
        self._measure_side_flag = None
        self._metrics_drain = "json"
        self._output_schema = {}

    def set_measure_at_filter(self, lambda_src: RustCode) -> Self:
        """Set the rule for event triggered measurement."""
        self._measure_at_event_lambda = lambda_src
        return self

    def enable_measure_for_event(self) -> Self:
        """Enable measurement on every input event behavior."""
        self._measure_at_event_lambda = "|_| true"
        return self

    def disable_measure_for_event(self) -> Self:
        """Prevent measurement on any input event."""
        self._measure_at_event_lambda = "|_| false"
        return self

    def set_trigger_signal(self, signal: SignalBase) -> Self:
        """Set the measurement control signal.

        This signal will trigger a measurement when the value of the signal gets changed.
        """
        self._measure_on_edge = signal
        return self

    def set_limit_side_signal(self, signal: SignalBase) -> Self:
        """Configure which one-sided limit should be used for measurements.

        Normally, LSP uses the right limit for measurements.
        While for some special case, for example, the summary for the end of a session, we should use the left limit
        semantics. And this is the signal that switches the limit-side semantics during the runtime.
        """
        self._measure_side_flag = signal
        return self

    def set_metrics_drain(self, fmt: str) -> Self:
        """Configure what format we want the LSP system produce.

        Note: Currently JSON is the only valid option.
        """
        self._metrics_drain = fmt
        return self

    def add_metric(self, key: str, measurement: MeasurementBase, typename: RustCode = COMPILER_INFERABLE_TYPE) -> Self:
        """Declare a metric for output."""
        if typename == COMPILER_INFERABLE_TYPE:  # if this type is unknown and inferable
            typename = measurement.get_rust_type_name()
        if typename == COMPILER_INFERABLE_TYPE:  # if this type can't be inferred
            logging.error("Please provide the type name for this metric.")
            logging.info("Consider call `.annotate_type(<type-name>)` to manually annotate signal's type.")
            raise Exception(f"Missing type name for the metric {key}.")
        self._output_schema[key] = {
            "source": measurement.get_description(),
            "type": typename
        }
        return self

    def to_dict(self) -> dict[str, Any]:
        """Dump the measurement policy into a dictionary that can be JSONified."""
        ret = {
            "measure_at_event_filter": self._measure_at_event_lambda,
            "metrics_drain": self._metrics_drain,
            "output_schema": self._output_schema,
        }
        if self._measure_on_edge is not None:
            ret["measure_trigger_signal"] = self._measure_on_edge.get_description()
        if self._measure_side_flag is not None:
            ret["measure_left_side_limit_signal"] = self._measure_side_flag.get_description()
        return ret


def _make_measurement_configuration():
    config = _MeasurementConfiguration()
    return lambda: config


measurement_config = _make_measurement_configuration()
