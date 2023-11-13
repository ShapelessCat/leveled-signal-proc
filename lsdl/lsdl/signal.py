from abc import ABC
from typing import Optional

from .lsp_model_component import LeveledSignalProcessingModelComponentBase
from .measurement import MeasurementBase


class SignalBase(LeveledSignalProcessingModelComponentBase, ABC):
    def map(self, bind_var: str, lambda_src: str) -> 'SignalBase':
        """Shortcut to apply a signal mapper on current signal.

        It allows applying Rust lambda on current signal.
        The result is also a leveled signal.
        """
        from .signal_processors import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)

    def count_changes(self) -> 'SignalBase':
        """Creates a new signal that counts the number of changes for current signal.

        The result is a leveled signal.
        Note: this is actually a shortcut for particular usage of accumulator signal processor.
        """
        from .signal_processors import Accumulator
        from .const import Const
        return Accumulator(self, Const(1))

    def has_been_true(self, duration=-1) -> 'SignalBase':
        """Shortcut for `has_been_true` module.

        Checks if the boolean signal has ever becomes true, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has been true within `duration` amount of time.
        """
        from .modules import has_been_true
        return has_been_true(self, duration)

    def has_changed(self, duration=-1) -> 'SignalBase':
        """Shortcut for `has_changed` module.

        Checks if the signal has ever changed, and the result is a leveled signal.
        When `duration` is given, it checks if the signal has changed within `duration` amount of time.
        """
        from .modules import has_changed
        return has_changed(self, duration)

    def prior_different_value(self, scope: Optional['SignalBase'] = None) -> 'SignalBase':
        return self.prior_value(self, scope)

    def prior_value(self, clock: Optional['SignalBase'] = None, scope: Optional['SignalBase'] = None, window=1, initial_value=None) -> 'SignalBase':
        from .schema import MappedInputMember
        from .signal_processors import StateMachineBuilder
        if clock is None:
            if isinstance(self, MappedInputMember):
                clock = self.clock()
            else:
                raise ValueError(
                    """Please
                       1. either provide a signal as the required clock
                       2. or make sure the `self` is a `MappedInputMember` instance, which has the `clock()` method"""
                )
        ty = self.get_rust_type_name()
        initial_value = initial_value or f"Default::default()"
        builder = StateMachineBuilder(data = self, clock = clock)\
            .init_state(f'(std::collections::VecDeque::with_capacity({window}), Default:default())')\
            .transition_fn(f"""
                |(q, _): &(std::collections::VecDeque<{ty}>, {ty}), data: &{ty}| {{
                    let mut to_output = {initial_value};
                    let mut q_cloned = q.clone();
                    if q_cloned.len() == {window} {{
                        to_output = q_cloned.pop_front().unwrap();
                    }}
                    q_cloned.push_back(data.clone());
                    (q_cloned, to_output.clone())
                }}
            """)
        if scope is not None:
            builder.scoped(scope)
        return builder.build().annotate_type(f"(std::collections::VecDeque<{ty}>, {ty})").map(
            bind_var='(_, ret)',
            lambda_src='ret.clone()'
        ).annotate_type(self.get_rust_type_name())

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

    def __eq__(self, other) -> 'SignalBase':
        return self._bin_op(other, "==", "bool")

    def __and__(self, other) -> 'SignalBase':
        return self._bin_op(other, "&&", "bool")

    def __or__(self, other) -> 'SignalBase':
        return self._bin_op(other, "||", "bool")

    def __xor__(self, other) -> 'SignalBase':
        return self._bin_op(other, "^", "bool")

    def __invert__(self) -> 'SignalBase':
        return self._bin_op(True, "^", "bool")

    def __lt__(self, other) -> 'SignalBase':
        return self._bin_op(other, "<", "bool")

    def __gt__(self, other) -> 'SignalBase':
        return self._bin_op(other, ">", "bool")

    def __le__(self, other) -> 'SignalBase':
        return self._bin_op(other, "<=", "bool")

    def __ge__(self, other) -> 'SignalBase':
        return self._bin_op(other, ">=", "bool")

    def __add__(self, other) -> 'SignalBase':
        return self._bin_op(other, "+", self.get_rust_type_name())

    def __sub__(self, other) -> 'SignalBase':
        return self._bin_op(other, "-", self.get_rust_type_name())

    def __mul__(self, other) -> 'SignalBase':
        return self._bin_op(other, "*", self.get_rust_type_name())

    def __div__(self, other) -> 'SignalBase':
        return self._bin_op(other, "/", self.get_rust_type_name())

    def measure_duration_true(self, scope_signal: Optional['SignalBase'] = None) -> 'MeasurementBase':
        """Measures the total duration whenever this boolean signal is true.

        It returns a measurement.
        When `scope_signal` is given, it resets the duration to 0 when the `scope_signal` becomes a different level.
        """
        from .measurements import DurationTrue
        return DurationTrue(self, scope_signal=scope_signal)

    def measure_duration_since_true(self) -> 'MeasurementBase':
        """Measures the duration when this boolean signal has been true most recently.

        When the boolean signal is false, the output of the measurement is constantly 0.
        """
        from .measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)

    def peek(self) -> 'MeasurementBase':
        """Returns the measurement that peek the latest value for the given signal."""
        from .measurements import Peek
        return Peek(self)
