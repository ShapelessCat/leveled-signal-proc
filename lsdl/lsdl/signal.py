

class LeveledSignalBase(object):
    def __init__(self):
        pass
    def get_id(self):
        raise NotImplementedError()
    def get_rust_type_name(self) -> str:
        raise NotImplementedError()
    def map(self, bind_var: str, lambda_src: str):
        from lsdl.component import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)
    def count_changes(self):
        from lsdl.component import ValueChangeCounter
        return ValueChangeCounter(self)
    def measure_duration_true(self):
        from lsdl.component import DurationTrue
        return DurationTrue(self)
    def _bin_op(self, other, op):
        from lsdl.component import SignalMapper
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
    