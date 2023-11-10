from abc import ABC
from typing import Optional, Self

from .lsp_model_component import LeveledSignalProcessingModelComponentBase


class SignalBase(LeveledSignalProcessingModelComponentBase, ABC):
    def map(self, bind_var: str, lambda_src: str) -> Self:
        """Shortcut to apply a signal mapper on current signal.

        It allows applying Rust lambda on current signal.
        The result is also a leveled signal.
        """
        from .signal_processors import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)

    def count_changes(self) -> Self:
        """Creates a new signal that counts the number of changes for current signal.

        The result is a leveled signal.
        Note: this is actually a shortcut for particular usage of accumulator signal processor.
        """
        from .signal_processors import Accumulator
        from .const import Const
        return Accumulator(self, Const(1))

    def has_been_true(self, duration = -1) -> Self:
        """Shortcut for `has_been_true` module.

        Checks if the boolean signal has ever becomes true, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has been true within `duration` amount of time.
        """
        from .modules import has_been_true
        return has_been_true(self, duration)

    def has_changed(self, duration = -1) -> Self:
        """Shortcut for `has_changed` module.

        Checks if the signal has ever changed, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has changed within `duration` amount of time.
        """
        from .modules import has_changed
        return has_changed(self, duration)

    def prior_different_value(self, scope: Optional[Self] = None) -> Self:
        return self.prior_value(self, scope)

    def prior_value(self, clock: Optional[Self] = None, scope: Optional[Self] = None) -> Self:
        from .signal_processors import StateMachineBuilder
        if clock is None:
            clock = self.clock()
        ty = self.get_rust_type_name()
        builder = StateMachineBuilder(data = self, clock = clock)\
            .transition_fn(f'|(_, current): &({ty}, {ty}), data : &{ty}| (current.clone(), data.clone())')
        if scope is not None:
            builder.scoped(scope)
        return builder.build().annotate_type(f"({ty}, {ty})").map(
            bind_var = '(ret, _)',
            lambda_src = 'ret.clone()'
        ).annotate_type(self.get_rust_type_name())

    def _bin_op(self, other, op, typename=None) -> Self:
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
                lambda_src=f"*lhs {op} {Const(other).get_rust_instant_value()}",
                upstream=self
            )
        if typename is not None:
            ret.annotate_type(typename)
        return ret

    def __eq__(self, other) -> Self:
        return self._bin_op(other, "==", "bool")

    def __and__(self, other) -> Self:
        return self._bin_op(other, "&&", "bool")

    def __or__(self, other) -> Self:
        return self._bin_op(other, "||", "bool")

    def __xor__(self, other) -> Self:
        return self._bin_op(other, "^", "bool")

    def __invert__(self) -> Self:
        return self._bin_op(True, "^", "bool")

    def __lt__(self, other) -> Self:
        return self._bin_op(other, "<", "bool")

    def __gt__(self, other) -> Self:
        return self._bin_op(other, ">", "bool")

    def __le__(self, other) -> Self:
        return self._bin_op(other, "<=", "bool")

    def __ge__(self, other) -> Self:
        return self._bin_op(other, ">=", "bool")

    def __add__(self, other) -> Self:
        return self._bin_op(other, "+", self.get_rust_type_name())

    def __sub__(self, other) -> Self:
        return self._bin_op(other, "-", self.get_rust_type_name())

    def __mul__(self, other) -> Self:
        return self._bin_op(other, "*", self.get_rust_type_name())

    def __div__(self, other) -> Self:
        return self._bin_op(other, "/", self.get_rust_type_name())

    def measure_duration_true(self, scope_signal: Optional[Self] = None) -> 'BuiltinMeasurementComponentBase':
        """Measures the total duration whenever this boolean signal is true.

        It returns a measurement.
        When `scope_signal` is given, it resets the duration to 0 when the `scope_signal` becomes a different level.
        """
        from .measurements import DurationTrue
        return DurationTrue(self, scope_signal = scope_signal)

    def measure_duration_since_true(self) -> 'BuiltinMeasurementComponentBase':
        """Measures the duration when this boolean signal has been true most recently.

        When the boolean signal is false, the output of the measurement is constantly 0.
        """
        from .measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)

    def peek(self) -> 'BuiltinMeasurementComponentBase':
        """Returns the measurement that peek the latest value for the given signal.
        """
        from .measurements import Peek
        return Peek(self)