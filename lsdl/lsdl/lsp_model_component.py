from abc import ABC

from .debug_info import DebugInfo


class LeveledSignalProcessingModelComponentBase(ABC):
    """A leveled signal processing model component base class.

    See LSP documentation for details about leveled signal definition.
    """
    def __init__(self):
        self.debug_info = DebugInfo()

    def get_id(self):
        """Get the IR description of the signal."""
        raise NotImplementedError()

    def get_rust_type_name(self) -> str:
        """Get the rust declaration for the type of this signal."""
        raise NotImplementedError()
