from abc import ABC
from typing import Optional, final

from .lsp_model_component import LeveledSignalProcessingModelComponentBase
from .measurement import MeasurementBase
from .rust_code import COMPILER_INFERABLE_TYPE, RustCode, RUST_DEFAULT_VALUE


class SignalBase(LeveledSignalProcessingModelComponentBase, ABC):
    @final
    def map(self, bind_var: str, lambda_src: str) -> 'SignalBase':
        """Shortcut to apply a signal mapper on current signal.

        It allows applying Rust lambda on current signal.
        The result is also a leveled signal.
        """
        from .signal_processors import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)

    @final
    def count_changes(self) -> 'SignalBase':
        """Creates a new signal that counts the number of changes for current signal.

        The result is a leveled signal.
        Note: this is actually a shortcut for particular usage of accumulator signal processor.
        """
        from .signal_processors import Accumulator
        from .const import Const
        return Accumulator(self, Const(1))

    @final
    def has_been_true(self, duration=-1) -> 'SignalBase':
        """Shortcut for `has_been_true` module.

        Checks if the boolean signal has ever becomes true, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has been true within `duration`.
        """
        from .modules import has_been_true
        return has_been_true(self, duration)

    @final
    def has_changed(self, duration=-1) -> 'SignalBase':
        """Shortcut for `has_changed` module.

        Checks if the signal has ever changed, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has changed within `duration`.
        """
        from .modules import has_changed
        return has_changed(self, duration)

    @final
    def prior_event(self, window_size=1, init_value=None) -> 'SignalBase':
        from .signal_processors import SlidingWindow
        if not init_value:
            init_value = RUST_DEFAULT_VALUE
        sw = SlidingWindow(
            clock=self,
            data=self,
            window_size=window_size,
            init_value=init_value,
            emit_fn='|_, data| data.clone()'
        )
        return sw.annotate_type(self.get_rust_type_name())

    def epoch_seconds(self) -> 'SignalBase':
        from .signal_processors import SignalGenerator
        return SignalGenerator(lambda_src="|t| (t, 0)").annotate_type("u64")

    def moving_average(self, window_size=1, init_value=0) -> 'SignalBase':
        from .signal_processors import SlidingWindow
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
        from .signal_processors import StateMachineBuilder
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
        builder = StateMachineBuilder(data=self, clock=clock)\
            .transition_fn(
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
        from .signal_processors import SignalMapper
        from .const import Const
        if isinstance(other, SignalBase):
            ret = SignalMapper(
                bind_var="(lhs, rhs)",
                lambda_src=f"*lhs {op} *rhs",
                upstream=[self, other]
            )
        else:
            ret = SignalMapper(
                bind_var="lhs",
                lambda_src=f"*lhs {op} {Const(other).rust_constant_value}",
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
                   typename: RustCode = COMPILER_INFERABLE_TYPE) -> 'SignalBase':
        from .modules import add_metric
        return add_metric(self, key, typename)

    @final
    def measure_linear_change(self) -> MeasurementBase:
        """Measures the change of some value, which has a fixed change rate for each level.

        It returns a measurement.
        The input leveled signal must be a change rate.
        When `scope_signal` is given, it resets the change to 0 when the `scope_signal` becomes
        a different level.
        """
        from .measurements import LinearChange
        return LinearChange(self)

    @final
    def measure_duration_true(self) -> MeasurementBase:
        """Measures the total duration whenever this boolean signal is true.

        It returns a measurement.
        When `scope_signal` is given, it resets the duration to 0 when the `scope_signal` becomes
        a different level.
        """
        from .measurements import DurationTrue
        return DurationTrue(self)

    @final
    def measure_duration_since_true(self) -> MeasurementBase:
        """Measures the duration when this boolean signal has been true most recently.

        When the boolean signal is false, the output of the measurement is constantly 0.
        """
        from .measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)

    @final
    def measure_duration_since_last_level(self) -> MeasurementBase:
        """Measures the duration since last level change happened.

        When there is no input signal happens, the output of the measurement is constantly 0.
        """
        from .measurements import DurationSinceLastLevel
        return DurationSinceLastLevel(self)

    @final
    def peek(self) -> MeasurementBase:
        """Returns the measurement that peek the latest value for the given signal."""
        from .measurements import Peek
        return Peek(self)

    @final
    def peek_timestamp(self) -> MeasurementBase:
        """Returns the current measurement timestamp for the given signal."""
        from .measurements import PeekTimestamp
        return PeekTimestamp(self)
