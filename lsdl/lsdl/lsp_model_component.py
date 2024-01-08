from abc import ABC
from typing import Any, Self

from .debug_info import DebugInfo
from .rust_code import RustCode


class LeveledSignalProcessingModelComponentBase(ABC):
    """A leveled signal processing model component base class.

    See LSP documentation for details about leveled signal definition.
    """
    def __init__(self, rust_type: RustCode):
        self._rust_type = rust_type
        self._is_moved = False
        self.debug_info = DebugInfo()

    @property
    def is_moved(self) -> bool:
        return self._is_moved

    @is_moved.setter
    def is_moved(self, value: bool):
        self._is_moved = value

    def annotate_type(self, type_name: RustCode) -> Self:
        self._rust_type = type_name
        return self

    # final
    def get_rust_type_name(self) -> RustCode:
        """Get the rust declaration for the type of this signal."""
        return self._rust_type

    def get_description(self) -> dict[str, Any]:
        """Get the description of current component."""
        raise NotImplementedError()
