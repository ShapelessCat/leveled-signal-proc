from abc import ABC

from .lsp_model_component import LeveledSignalProcessingModelComponentBase
from .rust_code import COMPILER_INFERABLE_TYPE, RustCode


# TODO: Add measurement combinators here
class MeasurementBase(LeveledSignalProcessingModelComponentBase, ABC):
    def add_metric(self, key: RustCode, typename: RustCode = COMPILER_INFERABLE_TYPE) -> 'MeasurementBase':
        from .modules import add_metric
        return add_metric(self, key, typename)
