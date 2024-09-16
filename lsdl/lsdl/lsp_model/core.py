import configparser
import os
from abc import ABC
from typing import Any, Optional, Self, final

from ..debug_info import DebugInfo
from ..rust_code import COMPILER_INFERABLE_TYPE, RustCode


class LeveledSignalProcessingModelComponentBase(ABC):
    """A leveled signal processing model component base class.

    See LSP documentation for details about leveled signal definition.
    """
    def __init__(self, rust_type: RustCode):
        self._rust_type = rust_type
        self.debug_info = DebugInfo()

    def add_metric(self,
                   key: RustCode,
                   typename: RustCode = COMPILER_INFERABLE_TYPE,
                   need_interval_metric: bool = False,
                   interval_metric_name: Optional[str] = None) -> Self:
        """Register the leveled signal as a metric.

        The registered metric results will present in the output data structure.

        Note:
        to register the type, the leveled signal should have a known type, otherwise, it's an error.
        """
        raise NotImplementedError()

    def annotate_type(self, type_name: RustCode) -> Self:
        self._rust_type = type_name
        return self

    @final
    def get_rust_type_name(self) -> RustCode:
        """Get the rust declaration for the type of this signal."""
        return self._rust_type

    def get_description(self) -> dict[str, Any]:
        """Get the description of current component."""
        raise NotImplementedError()


__config = configparser.ConfigParser()
__current_file_path = os.path.dirname(os.path.abspath(__file__))
__config.read(f"{__current_file_path}/rust_keywords.ini")
__strict_and_reserved_rust_keywords = {*__config['strict'].values(), *__config['reserved'].values()}


def _validate_rust_identifier(identifier: str) -> None:
    """Check if an identifier is a restricted legal Rust identifier.

    For implementation simplicity, only a C-style identifier that is not a Rust
    strict/reserved keyword is allowed.

    NOTE:
    We don't need to support all possible legal Rust identifiers. These identifiers are
    used as metric names, and C-style identifiers are enough for this use.
    """
    import re
    regex = '^[A-Za-z_][A-Za-z0-9_]*$'
    if not re.match(regex, identifier) or identifier in __strict_and_reserved_rust_keywords:
        raise Exception(f"{identifier} is not a simple and legal Rust identifier!")


class SignalBase(LeveledSignalProcessingModelComponentBase, ABC):
    __CMP_OP = ["==", "<", ">", "<=", ">="]

    @final
    def map(self, bind_var: str, lambda_src: str) -> 'SignalBase':
        """Shortcut to apply a signal mapper on current signal.

        It allows applying Rust lambda on current signal.
        The result is also a leveled signal.
        """
        from ..processors import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)

    @final
    def count_changes(self) -> 'SignalBase':
        """Creates a new signal that counts the number of changes for current signal.

        The result is a leveled signal.
        Note: this is actually a shortcut for particular usage of accumulator signal processor.
        """
        from ..processors import Accumulator, Const
        return Accumulator(self, Const(1))

    @final
    def has_been_true(self, duration=-1) -> 'SignalBase':
        """Checks if the boolean signal has ever becomes true.

        When `duration` is given, it checks if the signal has been true within `duration`.

        Note:
        `duration` can be either an integer as number of nanoseconds or a string of "<value><unit>".
        For example, "100ms", "2h", etc...
        """
        from .internal import normalize_duration
        from ..processors import Const, LevelTriggeredLatch
        return LevelTriggeredLatch(
            data=Const(True),
            control=self,
            forget_duration=normalize_duration(duration)
        )

    @final
    def has_changed(self, duration=-1) -> 'SignalBase':
        """Checks if the signal has ever changed.

        Return a leveled signal.
        When `duration` is given, it checks if the signal has changed within `duration`.

        Note:
        `duration` can be either an integer as number of nanoseconds or a string of "<value><unit>".
        For example, "100ms", "2h", etc...
        """
        from .internal import normalize_duration
        from ..processors import Const, EdgeTriggeredLatch
        return EdgeTriggeredLatch(
            control=self,
            data=Const(True),
            forget_duration=normalize_duration(duration)
        )

    def moving_average(self, window_size=1, init_value=0) -> 'SignalBase':
        from ..processors import SlidingWindow
        ty = self.get_rust_type_name()
        sw = SlidingWindow(
            clock=self,
            data=self,
            window_size=window_size,
            init_value=init_value,
            emit_fn=f'''
                |(q, _): (&std::collections::VecDeque<{ty}>, &{ty})|
                q.iter().fold(0, |a, x| a + x) as f64 / q.len() as f64
            '''
        )
        return sw.annotate_type('f64')

    @final
    def prior_different_value(self, scope: Optional['SignalBase'] = None) -> 'SignalBase':
        return self.prior_value(self, scope)

    @final
    def prior_value(self,
                    clock: Optional['SignalBase'] = None,
                    scope: Optional['SignalBase'] = None) -> 'SignalBase':
        from .schema import MappedInputMember
        from ..processors import StateMachineBuilder
        if clock is None:
            if isinstance(self, MappedInputMember):
                clock = self.clock()
            else:
                raise ValueError(
                    """Please
                       1. either provide a signal as the required clock
                       2. or make sure the `self` is a `MappedInputMember` instance,
                          which has the `clock()` method"""
                )
        ty = self.get_rust_type_name()
        builder = StateMachineBuilder(data=self, clock=clock).transition_fn(
            f'|(_, current): &({ty}, {ty}), data : &{ty}| (current.clone(), data.clone())'
        )
        if scope is not None:
            builder.scoped(scope)
        return builder.build().annotate_type(f"({ty}, {ty})").map(
            bind_var='(ret, _)',
            lambda_src='ret.clone()'
        ).annotate_type(self.get_rust_type_name())

    @final
    def _bin_op(self, other, op, typename=None) -> 'SignalBase':
        from ..processors import Const, SignalMapper
        if isinstance(other, SignalBase):
            ret = SignalMapper(
                bind_var="(lhs, rhs)",
                lambda_src=f"*lhs {op} *rhs",
                upstream=[self, other]
            )
        else:
            bind_var="lhs"
            is_cmp_string = op in SignalBase.__CMP_OP and self.get_rust_type_name() == "String"
            bind_var_in_use = f"{bind_var}.as_str()" if is_cmp_string else f"*{bind_var}"
            ret = SignalMapper(
                bind_var,
                lambda_src=f"{bind_var_in_use} {op} {Const(other, need_owned=False).rust_constant_value}",
                upstream=self
            )
        if typename is not None:
            ret.annotate_type(typename)
        return ret

    @final
    def __eq__(self, other) -> 'SignalBase':
        return self._bin_op(other, "==", "bool")

    @final
    def __and__(self, other) -> 'SignalBase':
        return self._bin_op(other, "&&", "bool")

    @final
    def __or__(self, other) -> 'SignalBase':
        return self._bin_op(other, "||", "bool")

    @final
    def __xor__(self, other) -> 'SignalBase':
        return self._bin_op(other, "^", "bool")

    @final
    def __invert__(self) -> 'SignalBase':
        return self._bin_op(True, "^", "bool")

    @final
    def __lt__(self, other) -> 'SignalBase':
        return self._bin_op(other, "<", "bool")

    @final
    def __gt__(self, other) -> 'SignalBase':
        return self._bin_op(other, ">", "bool")

    @final
    def __le__(self, other) -> 'SignalBase':
        return self._bin_op(other, "<=", "bool")

    @final
    def __ge__(self, other) -> 'SignalBase':
        return self._bin_op(other, ">=", "bool")

    @final
    def __add__(self, other) -> 'SignalBase':
        return self._bin_op(other, "+", self.get_rust_type_name())

    @final
    def __sub__(self, other) -> 'SignalBase':
        return self._bin_op(other, "-", self.get_rust_type_name())

    @final
    def __mul__(self, other) -> 'SignalBase':
        return self._bin_op(other, "*", self.get_rust_type_name())

    @final
    def __div__(self, other) -> 'SignalBase':
        return self._bin_op(other, "/", self.get_rust_type_name())

    @final
    def add_metric(self,
                   key: RustCode,
                   typename: RustCode = COMPILER_INFERABLE_TYPE,
                   need_interval_metric: bool = False,
                   interval_metric_name: Optional[str] = None) -> 'SignalBase':
        _validate_rust_identifier(key)
        from ..config import measurement_config
        measurement_config().add_metric(key, self.peek(), typename, need_interval_metric,
                                        interval_metric_name)
        return self

    @final
    def measure_linear_change(self) -> 'MeasurementBase':
        """Measures the change of some value, which has a fixed change rate for each level.

        It returns a measurement.
        The input leveled signal must be a change rate.
        When `scope_signal` is given, it resets the change to 0 when the `scope_signal` becomes
        a different level.
        """
        from ..measurements import LinearChange
        return LinearChange(self)

    @final
    def measure_duration_true(self) -> 'MeasurementBase':
        """Measures the total duration whenever this boolean signal is true.

        It returns a measurement.
        When `scope_signal` is given, it resets the duration to 0 when the `scope_signal` becomes
        a different level.
        """
        from ..measurements import DurationTrue
        return DurationTrue(self)

    @final
    def measure_duration_since_true(self) -> 'MeasurementBase':
        """Measures the duration when this boolean signal has been true most recently.

        When the boolean signal is false, the output of the measurement is constantly 0.
        """
        from ..measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)

    @final
    def measure_duration_of_current_level(self) -> 'MeasurementBase':
        """Measures the duration of current level.

        When there is no input signal happens, the output of the measurement is constantly 0.
        """
        from ..measurements import DurationOfCurrentLevel
        return DurationOfCurrentLevel(self)

    @final
    def peek(self) -> 'MeasurementBase':
        """Returns the measurement that peek the latest value for the given signal."""
        from ..measurements import Peek
        return Peek(self)

    @final
    def peek_timestamp(self, apply_builtin_formatter=False) -> 'MeasurementBase':
        """Returns the current measurement timestamp for the given signal."""
        from ..measurements import PeekTimestamp
        peek_ts = PeekTimestamp(self)
        if apply_builtin_formatter:
            # Assume the input `nano_seconds` is a UTC timestamp
            (lambda_param, lambda_body) = PeekTimestamp.BUILTIN_DATETIME_FORMATTER
            mapped_peak_ts = peek_ts.map(lambda_param, lambda_body)
            mapped_peak_ts.annotate_type('String')
            return mapped_peak_ts
        else:
            return peek_ts


class MeasurementBase(LeveledSignalProcessingModelComponentBase, ABC):
    @final
    def add_metric(self,
                   key: RustCode,
                   typename: RustCode = COMPILER_INFERABLE_TYPE,
                   need_interval_metric: bool = False,
                   interval_metric_name: Optional[str] = None) -> 'MeasurementBase':
        _validate_rust_identifier(key)
        from ..config import measurement_config
        measurement_config().add_metric(key, self, typename, need_interval_metric,
                                        interval_metric_name)
        return self

    @final
    def map(self, bind_var: str, lambda_src: str) -> 'MeasurementBase':
        """Shortcut to apply a measurement mapper on current measurement.

        It allows applying Rust lambda on current measurement result.
        """
        from ..measurements.combinators.mapper import MappedMeasurement
        return MappedMeasurement(bind_var, lambda_src, self)

    @final
    def scope(self, scope_signal: 'SignalBase') -> 'MeasurementBase':  # noqa: F821
        """Shortcut to reset a measurement based on a given signal."""
        from ..measurements.combinators.scope import ScopedMeasurement
        return ScopedMeasurement(scope_signal, self)

    @final
    def combine(self,
                bind_var0: str, bind_var1: str, lambda_src: str,
                other: 'MeasurementBase') -> 'MeasurementBase':
        """Shortcut to combine two measurements by provided lambda."""
        from ..measurements.combinators.binary import BinaryCombinedMeasurement
        return BinaryCombinedMeasurement(bind_var0, bind_var1, lambda_src, self, other)
