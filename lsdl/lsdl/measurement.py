from abc import ABC

from .lsp_model_component import LeveledSignalProcessingModelComponentBase
from .rust_code import COMPILER_INFERABLE_TYPE, RustCode


# TODO: Add measurement combinators here
class MeasurementBase(LeveledSignalProcessingModelComponentBase, ABC):
    def add_metric(self, key: RustCode, typename: RustCode = COMPILER_INFERABLE_TYPE) -> 'MeasurementBase':
        from .modules import add_metric
        return add_metric(self, key, typename)

    # def map(self, bind_var: str, lambda_src: str) -> 'MeasurementBase':
    #     """Shortcut to apply a signal mapper on current signal.
    #
    #     It allows applying Rust lambda on current signal.
    #     The result is also a leveled signal.
    #     """
    #     from .measurements import MappedPeekTimestamp
    #     return MappedPeekTimestamp(bind_var, lambda_src, self)
