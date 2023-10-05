from lsdl.debug_info import DebugInfo

class LeveledSignalBase(object):
    def __init__(self):
        self._debug_info = DebugInfo()
    def get_id(self):
        raise NotImplementedError()
    def get_rust_type_name(self) -> str:
        raise NotImplementedError()
    def map(self, bind_var: str, lambda_src: str):
        from lsdl.signal_processors import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)
    def count_changes(self):
        from lsdl.signal_processors import Accumulator 
        from lsdl.const import Const
        return Accumulator(self, Const(1))
    def measure_duration_true(self, scope_signal = None):
        from lsdl.measurements import DurationTrue
        return DurationTrue(self, scope_signal = scope_signal)
    def measure_duration_since_true(self):
        from lsdl.measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)
    def measure_change(self, control_signal):
        from lsdl.measurements import DiffSinceCurrentLevel
        return DiffSinceCurrentLevel(control = control_signal, data = self)
    def peek(self):
        from lsdl.measurements import PeekValue
        return PeekValue(self)
    def _bin_op(self, other, op, typename = None):
        from lsdl.signal_processors import SignalMapper
        from lsdl.const import Const
        if isinstance(other, LeveledSignalBase):
            ret = SignalMapper(
                bind_var="(lhs, rhs)",
                lambda_src="*lhs {op} *rhs".format(op = op),
                upstream=[self, other]
            )
        else:
            ret = SignalMapper(
                bind_var="lhs",
                lambda_src="*lhs {op} {other}".format(other=Const(other).get_rust_instant_value(), op = op),
                upstream = self
            )
        if typename is not None:
            ret._output_type = typename
        return ret
    def __eq__(self, other):
        return self._bin_op(other, "==", "bool")
    def __and__(self, other):
        return self._bin_op(other, "&&", "bool")
    def __or__(self, other):
        return self._bin_op(other, "||", "bool")
    def __xor__(self, other):
        return self._bin_op(other, "^", "bool")
    def __invert__(self):
        return self._bin_op(True, "^", "bool")
    def __lt__(self, other):
        return self._bin_op(other, "<", "bool")
    def __gt__(self, other):
        return self._bin_op(other, ">", "bool")
    def __le__(self, other):
        return self._bin_op(other, "<=", "bool")
    def __ge__(self, other):
        return self._bin_op(other, ">=", "bool")
    def __add__(self, other):
        return self._bin_op(other, "+")
    def __sub__(self, other):
        return self._bin_op(other, "-")
    def __mul__(self, other):
        return self._bin_op(other, "*")
    def __div__(self, other):
        return self._bin_op(other, "/")

class If(LeveledSignalBase):
    def __init__(self, cond_expr: LeveledSignalBase, then_expr: LeveledSignalBase, else_expr: LeveledSignalBase):
        from lsdl.signal_processors import SignalMapper
        super().__init__()
        self._cond = cond_expr
        self._then = then_expr
        self._else = else_expr
        self._inner = SignalMapper(
            bind_var = "(cond, then_expr, else_expr)",
            lambda_src = """if *cond { then_expr.clone() } else { else_expr.clone() }""",
            upstream = [self._cond, self._then, self._else]
        )
        if self._then.get_rust_type_name() == self._else.get_rust_type_name():
            self._inner._output_type = self._then.get_rust_type_name()
    def get_id(self):
        return self._inner.get_id()
    def get_rust_type_name(self) -> str:
        return self._inner.get_rust_type_name()
