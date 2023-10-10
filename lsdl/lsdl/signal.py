from lsdl.debug_info import DebugInfo


class LeveledSignalBase:
    """
        A leveled signal.
        See LSP documentation for details about leveled signal definition.
    """
    def __init__(self):
        self._debug_info = DebugInfo()
    def get_id(self):
        """
            Get the IR description of the signal
        """
        raise NotImplementedError()
    def get_rust_type_name(self) -> str:
        """
            Get the rust declaration for the type of this signal
        """
        raise NotImplementedError()
    def is_signal(self) -> bool:
        """
            Is this a signal or measurement ?
        """
        raise NotImplementedError()
    def has_been_true(self, duration = -1) -> 'LeveledSignalBase':
        """
            Shortcut for has_been_true module.
            Checks if the boolean signal has ever becomes true and returns the result as a leveled signal.
            When `duration` is given, it checks if the signal has been true within `duration` amount of time.
        """
        from lsdl.modules import has_been_true
        return has_been_true(self, duration)
    def has_changed(self, duration = -1) -> 'LeveledSignalBase':
        """
            Shortcut for has_changed module.
            Checks if the signal has ever changed and returns the result as a leveled signal.
            When `duration` is given, it checks if the signal has changed within `duration` amount of time.
        """
        from lsdl.modules import has_changed
        return has_changed(self, duration)
    def map(self, bind_var: str, lambda_src: str) -> 'LeveledSignalBase':
        """
            Shortcut to apply a signal mapper on current signal.
            It allows applying Rust lambda on current signal. The result is also a leveled signal.
        """
        from lsdl.signal_processors import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)

    def prior_different_value(self, scope: 'LeveledSignalBase' = None) -> 'LeveledSignalBase':
        return self.prior_value(self, scope)

    def prior_value(self, clock: 'LeveledSignalBase' = None, scope: 'LeveledSignalBase' = None) -> 'LeveledSignalBase':
        from lsdl.signal_processors import StateMachineBuilder
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

    def count_changes(self) -> 'LeveledSignalBase':
        """
            Creates a new signal that counts the number of changes for current signal.
            The result is a leveled signal.
            Note: this is actually a shortcut for particular usage of accumulator signal processor.
        """
        from lsdl.signal_processors import Accumulator
        from lsdl.const import Const
        return Accumulator(self, Const(1))
    def measure_duration_true(self, scope_signal = None) -> 'LeveledSignalBase':
        """
            Measures the total duration whenever this boolean signal is true.
            It returns a measurement.
            When `scope_signal` is given, it resets the duration to 0 when the `scope_signal` becomes a different level.
        """
        from lsdl.measurements import DurationTrue
        return DurationTrue(self, scope_signal = scope_signal)
    def measure_duration_since_true(self) -> 'LeveledSignalBase':
        """
            Measures the duration when this boolean signal has been true most recently.
            When the boolean signal is false, the output of the measurement is constantly 0.
        """
        from lsdl.measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)
    def peek(self) -> 'LeveledSignalBase':
        """
            Returns the measurement that peek the latest value for the given signal.
        """
        from lsdl.measurements import PeekValue
        return PeekValue(self)
    def add_metric(self, key, typename = "_") -> 'LeveledSignalBase':
        """
            Register the leveled signal as a metric that will present in the output metrics data structure.
            Note: to register the type, the leveled signal should have a known type, otherwise, it's an error.
        """
        from lsdl import measurement_config
        from lsdl.measurements import PeekValue
        if self.is_signal():
            measurement_config().add_metric(key, PeekValue(self), typename)
        else:
            measurement_config().add_metric(key, self, typename)
        return self
    def _bin_op(self, other, op, typename = None) -> 'LeveledSignalBase':
        from lsdl.signal_processors import SignalMapper
        from lsdl.const import Const
        if isinstance(other, LeveledSignalBase):
            ret = SignalMapper(
                bind_var="(lhs, rhs)",
                lambda_src=f"*lhs {op} *rhs",
                upstream=[self, other]
            )
        else:
            ret = SignalMapper(
                bind_var="lhs",
                lambda_src="*lhs {op} {other}".format(other=Const(other).get_rust_instant_value(), op = op),
                upstream = self
            )
        if typename is not None:
            ret.annotate_type(typename)
        return ret
    def __eq__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "==", "bool")
    def __and__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "&&", "bool")
    def __or__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "||", "bool")
    def __xor__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "^", "bool")
    def __invert__(self) -> 'LeveledSignalBase':
        return self._bin_op(True, "^", "bool")
    def __lt__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "<", "bool")
    def __gt__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, ">", "bool")
    def __le__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "<=", "bool")
    def __ge__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, ">=", "bool")
    def __add__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "+", self.get_rust_type_name())
    def __sub__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "-", self.get_rust_type_name())
    def __mul__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "*", self.get_rust_type_name())
    def __div__(self, other) -> 'LeveledSignalBase':
        return self._bin_op(other, "/", self.get_rust_type_name())


def _build_signal_mapper(cond: LeveledSignalBase,
                         then_branch: LeveledSignalBase,
                         else_branch: LeveledSignalBase) -> LeveledSignalBase:
    from lsdl.signal_processors import SignalMapper
    inner = SignalMapper(
        bind_var = "(cond, then_expr, else_expr)",
        lambda_src = """if *cond { then_expr.clone() } else { else_expr.clone() }""",
        upstream = [cond, then_branch, else_branch]
    )
    else_type = else_branch.get_rust_type_name()
    then_type = then_branch.get_rust_type_name()
    if then_type == "_":
        then_type = else_type
    elif else_type == "_":
        else_type = then_type

    if then_type == else_type:
        inner.annotate_type(then_type)
    return inner


class If(LeveledSignalBase):
    """The `if...then...else` expression for a leveled signal."""
    def __init__(self,
                 cond_expr: LeveledSignalBase,
                 then_expr: LeveledSignalBase,
                 else_expr: LeveledSignalBase):
        super().__init__()
        self._inner = _build_signal_mapper(cond_expr, then_expr, else_expr)

    def get_id(self):
        return self._inner.get_id()

    def get_rust_type_name(self) -> str:
        return self._inner.get_rust_type_name()

    def is_signal(self) -> bool:
        return True


class Cond(LeveledSignalBase):
    """The scheme `cond` style expression for a leveled signal."""
    def __init__(self,
                 first_branch: (LeveledSignalBase, LeveledSignalBase),
                 middle_branches: [(LeveledSignalBase, LeveledSignalBase)],
                 fallback_value: LeveledSignalBase):
        super().__init__()
        self._inner = _build_signal_mapper(*first_branch, fallback_value)
        while middle_branches:
            (cond, then_branch) = middle_branches.pop()
            self._inner = _build_signal_mapper(cond, then_branch, self._inner)

    def get_id(self):
        return self._inner.get_id()

    def get_rust_type_name(self) -> str:
        return self._inner.get_rust_type_name()

    def is_signal(self) -> bool:
        return True
