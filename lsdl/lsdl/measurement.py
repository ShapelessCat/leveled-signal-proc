from abc import ABC

from .lsp_model_component import LeveledSignalProcessingModelComponentBase
from .rust_code import COMPILER_INFERABLE_TYPE, RustCode


class MeasurementBase(LeveledSignalProcessingModelComponentBase, ABC):
    def add_metric(self, key: RustCode, typename: RustCode = COMPILER_INFERABLE_TYPE) -> 'MeasurementBase':
        from .modules import add_metric
        return add_metric(self, key, typename)

    def map(self, bind_var: str, lambda_src: str) -> 'MeasurementBase':
        """Shortcut to apply a measurement mapper on current measurement.
        It allows applying Rust lambda on current measurement result.
        The result is still a measurement.
        """
        from .measurements.combinators.mapper import MappedMeasurement
        return MappedMeasurement(bind_var, lambda_src, self)

    def scope(self, scope_signal: 'SignalBase') -> 'MeasurementBase':
        """Shortcut to reset a measurement based on a given signal.
        The result is still a measurement.
        """
        from .measurements.combinators.scope import ScopedMeasurement
        return ScopedMeasurement(scope_signal, self)

    def combine(self, bind_var0: str, bind_var1: str, lambda_src: str, other: 'MeasurementBase') -> 'MeasurementBase':
        """Shortcut to combine two measurements by provided lambda.
        The result is still a measurement.
        """
        from .measurements.combinators.binary import BinaryCombinedMeasurement
        return BinaryCombinedMeasurement(bind_var0, bind_var1, lambda_src, self, other)
