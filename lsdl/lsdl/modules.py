import re
from typing import Self, Optional, TypeVar

from . import validate_rust_identifier
from .const import Const
from .measurement import MeasurementBase
from .rust_code import COMPILER_INFERABLE_TYPE, RustCode
from .schema import MappedInputMember
from .signal import SignalBase
from .signal_processors import SignalMapper, Latch
from .signal_processors.latch import EdgeTriggeredLatch


def normalize_duration(duration: int | str) -> int:
    if isinstance(duration, str):
        value_str = re.search(r"\d+", duration).group(0)
        value_unit = duration[len(value_str):]
        value = int(value_str)
        match value_unit:
            case 's':
                duration = value * 1_000_000_000
            case 'ms':
                duration = value * 1_000_000
            case 'us':
                duration = value * 1_000
            case 'ns':
                duration = value
            case 'm':
                duration = value * 60_000_000_000
            case 'h':
                duration = value * 3_600_000_000_000
            case _:
                raise ValueError(f"Unknown duration unit: {value_unit}")
    return duration


def has_been_true(input_signal: SignalBase, duration: int | str = -1) -> SignalBase:
    """Checks if the boolean signal has ever becomes true.

    When `duration` is given, it checks if the signal has been true within `duration` amount of time.

    Note:
    `duration` can be either an integer as number of nanoseconds or a string of "<value><unit>".
    For example, "100ms", "2h", etc...
    """
    return Latch(
        data=Const(True),
        control=input_signal,
        forget_duration=normalize_duration(duration)
    )


def has_changed(input_signal: SignalBase, duration: int | str = -1) -> SignalBase:
    """Checks if the signal has ever changed.

    Return a leveled signal.
    When `duration` is given, it checks if the signal has changed within `duration` amount of time.

    Note:
    `duration` can be either an integer as number of nanoseconds or a string of "<value><unit>".
    For example, "100ms", "2h", etc...
    """
    return EdgeTriggeredLatch(
        control=input_signal,
        data=Const(True),
        forget_duration=normalize_duration(duration)
    )


def make_tuple(*args: SignalBase) -> SignalBase:
    """Make a tuple from multiple input signals."""
    return SignalMapper(
        bind_var="s",
        lambda_src="s.clone()",
        upstream=list(args)
    ).annotate_type(f'({",".join([arg.get_rust_type_name() for arg in args])})')


__T = TypeVar('__T', SignalBase, MeasurementBase)


def add_metric(component: __T, key: RustCode, typename: RustCode = COMPILER_INFERABLE_TYPE) -> __T:
    """Register the leveled signal as a metric.

    The registered metric results will present in the output data structure.

    Note:
    to register the type, the leveled signal should have a known type, otherwise, it's an error.
    """
    validate_rust_identifier(key)
    from . import measurement_config
    if isinstance(component, SignalBase):
        measurement_config().add_metric(key, component.peek(), typename)
    else:
        measurement_config().add_metric(key, component, typename)
    return component


class SignalFilterBuilder:
    """The builder class to build a signal filter.

    A signal filter is a filter that filters either the clock or value signal.
    It can filter with a Rust lambda function or a list of values.
    """
    def __init__(self, filter_signal: SignalBase, clock_signal: Optional[SignalBase] = None):
        self._filter_signal = filter_signal
        self._clock_signal = clock_signal
        self._filter_node = None
        if isinstance(filter_signal, MappedInputMember) and clock_signal is None:
            self._clock_signal = filter_signal.clock()
        self._filter_lambda = None

    def filter_fn(self, bind_var: str, lambda_body: str) -> Self:
        """Set the Rust lambda function that filters the signal."""
        self._filter_node = SignalMapper(
            bind_var=bind_var,
            upstream=self._filter_signal,
            lambda_src=lambda_body,
        )
        return self

    def filter_values(self, *args) -> Self:
        """Set the list of values that to filter."""
        values = args
        self._filter_node = (self._filter_signal == values[0])
        for value in values[1:]:
            self._filter_node = self._filter_node | (self._filter_signal == value)
        return self

    def filter_true(self) -> Self:
        """Filters the boolean signal when its values is true."""
        self._filter_node = self._filter_signal
        return self

    def then_filter(self, filter_signal: SignalBase) -> Self:
        """Builds the clock signal filter and then create a builder that performing cascade filtering."""
        signal_clock = self.build_clock_filter()
        ret = SignalFilterBuilder(filter_signal, signal_clock)
        if filter_signal.get_rust_type_name() == "bool":
            ret.filter_true()
        return ret

    def build_clock_filter(self) -> SignalBase:
        return Latch(
            data=self._clock_signal,
            control=self._filter_node
        )

    def build_value_filter(self) -> SignalBase:
        return Latch(
            data=self._filter_signal,
            control=self._filter_node
        )


class ScopeContext:
    def __init__(self, scope_level: SignalBase, epoch: SignalBase):
        self._scope = scope_level
        self._epoch = epoch

    def scoped(self, data: SignalBase, clock: SignalBase, default=None) -> SignalBase:
        from .signal_processors import EdgeTriggeredLatch, SignalMapper
        scope_starts = EdgeTriggeredLatch(control=self._scope, data=self._epoch)
        event_starts = EdgeTriggeredLatch(control=clock, data=self._epoch)
        return SignalMapper(
            bind_var="(sep, eep, signal)",
            lambda_src=f"""if *sep <= *eep {{ signal.clone() }} else {{
                {"Default::default()" if default is None else str(default)}
            }}""",
            upstream=[scope_starts, event_starts, data]
        ).annotate_type(data.get_rust_type_name())


def time_domain_fold(data: SignalBase, clock: Optional[SignalBase] = None, scope: Optional[SignalBase] = None,
                     fold_method="sum", init_state=None):
    if clock is None:
        clock = data
    from .signal_processors.state_machine import StateMachineBuilder
    data_type = data.get_rust_type_name()
    lambda_param = f"s: &{data_type}, d: &{data_type}"
    if fold_method == "sum":
        fold_method = f"|{lambda_param}| s.clone() + d.clone()"
        init_state = f"{data_type}::default()" if init_state is None else init_state
    elif fold_method == "min":
        fold_method = f"|{lambda_param}| s.clone().min(d.clone())"
        init_state = f"{data_type}::MAX" if init_state is None else init_state
    elif fold_method == "max":
        fold_method = f"|{lambda_param}| s.clone().max(d.clone())"
        init_state = f"{data_type}::MIN" if init_state is None else init_state
    elif fold_method == "and":
        fold_method = f"|{lambda_param}| *s && *d"
        init_state = f"true" if init_state is None else init_state
    elif fold_method == "or":
        fold_method = f"|{lambda_param}| *s || *d"
        init_state = f"false" if init_state is None else init_state
    builder = StateMachineBuilder(clock=clock, data=data)

    if init_state is not None:
        builder.init_state(init_state)

    builder.transition_fn(fold_method)

    if scope is not None:
        builder.scoped(scope)

    return builder.build().annotate_type(data.get_rust_type_name())
