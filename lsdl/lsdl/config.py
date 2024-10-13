import logging
import re
from dataclasses import dataclass
from typing import Any, Callable, Optional, Self, final

from .lsp_model.component_base import LspComponentBase
from .lsp_model.core import MeasurementBase, SignalBase
from .rust_code import COMPILER_INFERABLE_TYPE, RUST_DEFAULT_VALUE, RustCode

logging.basicConfig(encoding="utf-8", level=logging.INFO)


@final
class _ProcessingConfiguration:
    """The configuration for processing policy."""

    def __init__(self):
        self._merge_simultaneous_moments = True

    def set_merge_simultaneous_moments(self, should_merge: bool) -> Self:
        """Set the rule for handling simultaneous moments."""
        self._merge_simultaneous_moments = should_merge
        return self

    def to_dict(self) -> dict[str, Any]:
        """Dump the processing policy into a dictionary."""
        return {"merge_simultaneous_moments": self._merge_simultaneous_moments}


def _make_processing_configuration():
    config = _ProcessingConfiguration()
    return lambda: config


processing_config: Callable[[], _ProcessingConfiguration] = (
    _make_processing_configuration()
)


@dataclass(frozen=True)
class ResetSwitch:
    metric_name: RustCode
    initial_value: RustCode


@final
class _MeasurementConfiguration:
    """The configuration for measurement policy.

    In LSP, there are two method to trigger a measurement:
    1. triggered by an input event;
    2. triggered by a signal edge.
    """

    def __init__(self):
        self._measure_at_event_lambda = "|_| true"
        self._output_control_measurement_ids: list[str] = []
        self._measure_on_edge = None
        self._measure_side_flag = None
        self._metrics_drain = "json"
        self._output_schema = {}
        # for interval metrics
        self._complementary_output_schema = {}
        # (metric name, initial value)
        self._complementary_output_reset_switch: Optional[ResetSwitch] = None

    def set_measure_at_measurement_true(
        self, *lsp_components: LspComponentBase
    ) -> Self:
        """Set the rule for a single measurement triggered full measurement."""
        measurements: list[MeasurementBase] = []
        for c in lsp_components:
            match c:
                case MeasurementBase():
                    measurements.append(c)
                case SignalBase():
                    measurements.append(c.peek())
                case _:
                    raise TypeError("Expect a Measurement or Signal!")

        self._output_control_measurement_ids = [
            m.get_description()["id"] for m in measurements
        ]
        return self

    def set_measure_at_event_filter(self, lambda_src: RustCode) -> Self:
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
        While for some special case, for example, the summary for the end of a session, we should
        use the left limit semantics. And this is the signal that switches the limit-side semantics
        during the runtime.
        """
        self._measure_side_flag = signal
        return self

    def set_metrics_drain(self, fmt: str) -> Self:
        """Configure what format we want the LSP system produce.

        Note: Currently JSON is the only valid option.
        """
        self._metrics_drain = fmt
        return self

    def add_metric(
        self,
        key: str,
        measurement: MeasurementBase,
        typename: RustCode,
        need_interval_metric: bool,
        interval_metric_name: Optional[str],
    ) -> Self:
        """Declare a metric for output."""
        if typename == COMPILER_INFERABLE_TYPE:  # if this type is unknown and inferable
            typename = measurement.get_rust_type_name()
        if typename == COMPILER_INFERABLE_TYPE:  # if this type can't be inferred
            logging.error("Please provide the type name for this metric.")
            logging.info(
                "Consider call `.annotate_type(<type-name>)` to manually annotate signal's type."
            )
            raise Exception(f"Missing type name for the metric {key}.")
        self._output_schema[key] = {
            "source": measurement.get_description(),
            "type": typename,
        }
        if need_interval_metric:
            if interval_metric_name is None and not key.startswith("life"):
                raise Exception(
                    """This metric name doesn't start with 'life_navigation' or 'life_session', """
                    """and you also doesn't manually provide a interval metric name"""
                )
            metric_name = interval_metric_name or re.sub(
                r"^life_(navigation|session)", "interval", key
            )
            self._complementary_output_schema[metric_name] = {
                "type": typename,
                "source": measurement.get_description(),
                "source_metric_name": key,
            }
        return self

    def set_complementary_output_reset_switch(
        self, metric_name: RustCode, initial_value: RustCode = RUST_DEFAULT_VALUE
    ) -> Self:
        """Config the reset switch.

        The `initial_value` should be a value that MUSTN'T match the first `metric_name` value.
        With a well design, the default value is a good choice. However, this is not always true,
        sometimes people need to manually set it, and this is why we provide this API.
        """
        self._complementary_output_reset_switch = ResetSwitch(
            metric_name, initial_value
        )
        return self

    def to_dict(self) -> dict[str, Any]:
        """Dump the measurement policy into a dictionary."""
        ret: dict = {
            "measure_at_event_filter": self._measure_at_event_lambda,
            "metrics_drain": self._metrics_drain,
            "output_schema": self._output_schema,
        }
        if self._output_control_measurement_ids:
            ret["output_control_measurement_ids"] = self._output_control_measurement_ids

        if self._measure_on_edge is not None:
            ret["measure_trigger_signal"] = self._measure_on_edge.get_description()

        if self._measure_side_flag is not None:
            ret["measure_left_side_limit_signal"] = (
                self._measure_side_flag.get_description()
            )

        if self._complementary_output_schema:
            ret["complementary_output_config"] = {
                "schema": self._complementary_output_schema
            }
            if self._complementary_output_reset_switch is not None:
                key = self._complementary_output_reset_switch.metric_name
                ret["complementary_output_config"]["reset_switch"] = {
                    "metric_name": key,
                    "source": self._output_schema[key]["source"],
                    "initial_value": self._complementary_output_reset_switch.initial_value,
                }
        elif self._complementary_output_reset_switch is not None:
            # Warning message for developers. This is why we use class internal names, rather than
            # the names in output IR.
            message = " ".join(
                [
                    "Redundant config:",
                    "`self._complementary_output_schema` is empty, no interval metrics,",
                    "but `self._complementary_output_reset_switch` is set.",
                ]
            )
            logging.warning(message)
        return ret


def _make_measurement_configuration():
    config = _MeasurementConfiguration()
    return lambda: config


measurement_config: Callable[[], _MeasurementConfiguration] = (
    _make_measurement_configuration()
)
