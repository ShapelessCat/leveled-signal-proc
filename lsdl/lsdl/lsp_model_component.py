from abc import ABC
from typing import Self

from .debug_info import DebugInfo
from .rust_code import RustCode


class LeveledSignalProcessingModelComponentBase(ABC):
    """A leveled signal processing model component base class.

    See LSP documentation for details about leveled signal definition.
    """
    def __init__(self, rust_type: RustCode):
        self._rust_type = rust_type
        self.debug_info = DebugInfo()

    def annotate_type(self, type_name: RustCode) -> Self:
        self._rust_type = type_name
        return self

    # final
    def get_rust_type_name(self) -> RustCode:
        """Get the rust declaration for the type of this signal."""
        return self._rust_type

    def get_id(self):
        """Get the IR description of the signal."""
        raise NotImplementedError()
