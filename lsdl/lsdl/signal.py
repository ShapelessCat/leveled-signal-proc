from typing import Self
from .debug_info import DebugInfo


class LeveledSignalProcessingModelComponentBase:
    """A leveled signal processing model component base class.

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

    def _bin_op(self, other, op, typename = None) -> 'LeveledSignalProcessingModelComponentBase':
        from .signal_processors import SignalMapper
        from .const import Const
        if isinstance(other, LeveledSignalProcessingModelComponentBase):
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

    def __eq__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "==", "bool")

    def __and__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "&&", "bool")

    def __or__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "||", "bool")

    def __xor__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "^", "bool")

    def __invert__(self) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(True, "^", "bool")

    def __lt__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "<", "bool")

    def __gt__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, ">", "bool")

    def __le__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "<=", "bool")

    def __ge__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, ">=", "bool")

    def __add__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "+", self.get_rust_type_name())

    def __sub__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "-", self.get_rust_type_name())

    def __mul__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "*", self.get_rust_type_name())

    def __div__(self, other) -> 'LeveledSignalProcessingModelComponentBase':
        return self._bin_op(other, "/", self.get_rust_type_name())
