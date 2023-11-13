
from abc import ABC

from .lsp_model_component import LeveledSignalProcessingModelComponentBase


# TODO: Add measurement combinators here
class MeasurementBase(LeveledSignalProcessingModelComponentBase, ABC):
    pass
