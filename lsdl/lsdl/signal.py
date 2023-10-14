from typing import Self
from .debug_info import DebugInfo


class LeveledSignalBase:
    """A leveled signal.

    See LSP documentation for details about leveled signal definition.
    """
    def __init__(self):
        self._debug_info = DebugInfo()

    def get_id(self):
        """Get the IR description of the signal."""
        raise NotImplementedError()

    def get_rust_type_name(self) -> str:
        """Get the rust declaration for the type of this signal."""
        raise NotImplementedError()

    def map(self, bind_var: str, lambda_src: str) -> Self:
        """Shortcut to apply a signal mapper on current signal.

        It allows applying Rust lambda on current signal.
        The result is also a leveled signal.
        """
        from.signal_processors import SignalMapper
        return SignalMapper(bind_var, lambda_src, self)

    def count_changes(self) -> Self:
        """Creates a new signal that counts the number of changes for current signal.

        The result is a leveled signal.
        Note: this is actually a shortcut for particular usage of accumulator signal processor.
        """
        from .signal_processors import Accumulator
        from .const import Const
        return Accumulator(self, Const(1))

    def _bin_op(self, other, op, typename = None) -> 'LeveledSignalBase':
        from .signal_processors import SignalMapper
        from .const import Const
        if isinstance(other, LeveledSignalBase):
            ret = SignalMapper(
                bind_var="(lhs, rhs)",
                lambda_src=f"*lhs {op} *rhs",
                upstream=[self, other]
            )
        else:
            ret = SignalMapper(
                bind_var="lhs",
                lambda_src=f"*lhs {op} {Const(other).get_rust_instant_value()}",
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
    from .signal_processors import SignalMapper
    inner = SignalMapper(
        bind_var = "(cond, then_expr, else_expr)",
        lambda_src = "if *cond { then_expr.clone() } else { else_expr.clone() }",
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
