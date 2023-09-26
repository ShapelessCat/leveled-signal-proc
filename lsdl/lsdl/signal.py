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
    def measure_duration_true(self):
        from lsdl.measurements import DurationTrue
        return DurationTrue(self)
    def measure_duration_since_true(self):
        from lsdl.measurements import DurationSinceBecomeTrue
        return DurationSinceBecomeTrue(self)
    def measure_change(self, control_signal):
        from lsdl.measurements import DiffSinceCurrentLevel
        return DiffSinceCurrentLevel(control = control_signal, data = self)
    def _bin_op(self, other, op):
        from lsdl.signal_processors import SignalMapper
        from lsdl.const import Const
        if isinstance(other, LeveledSignalBase):
            return SignalMapper(
                bind_var="(lhs, rhs)",
                lambda_src="*lhs {op} *rhs".format(op = op),
                upstream=[self, other]
            )
        else:
            return SignalMapper(
                bind_var="lhs",
                lambda_src="*lhs {op} {other}".format(other=Const(other).get_rust_instant_value(), op = op),
                upstream = self
            )
    def __eq__(self, other):
        return self._bin_op(other, "==")
    def __and__(self, other):
        return self._bin_op(other, "&&")
    def __or__(self, other):
        return self._bin_op(other, "&&")
    def __xor__(self, other):
        return self._bin_op(other, "^")
    def __invert__(self):
        return self._bin_op(True, "^")
    def __lt__(self, other):
        return self._bin_op(other, "<")
    def __gt__(self, other):
        return self._bin_op(other, ">")
    def __le__(self, other):
        return self._bin_op(other, "<=")
    def __ge__(self, other):
        return self._bin_op(other, "<=")