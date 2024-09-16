from typing import Self, Optional, final

from ..lsp_model.core import SignalBase
from ..lsp_model.schema import MappedInputMember


@final
class SignalFilterBuilder:
    """The builder class to build a signal filter.

    A signal filter is a filter that filters either the clock or value signal.
    It can filter with a Rust lambda function or a list of values.
    """
    def __init__(self, filter_signal: SignalBase, clock_signal: Optional[SignalBase] = None):
        self._filter_signal = filter_signal
        self._clock_signal = clock_signal
        self._filter_node: SignalBase
        if isinstance(filter_signal, MappedInputMember) and clock_signal is None:
            self._clock_signal = filter_signal.clock()
        self._filter_lambda = None

    def filter_fn(self, bind_var: str, lambda_body: str) -> Self:
        """Set the Rust lambda function that filters the signal."""
        from ..processors import SignalMapper
        self._filter_node = SignalMapper(
            bind_var=bind_var,
            upstream=self._filter_signal,
            lambda_src=lambda_body,
        )
        return self

    def filter_values(self, *args) -> Self:
        """Set the list of values that to filter."""
        values = args
        self._filter_node = (self._filter_signal == values[0])
        for value in values[1:]:
            self._filter_node = self._filter_node | (self._filter_signal == value)
        return self

    def filter_true(self) -> Self:
        """Filters the boolean signal when its values is true."""
        self._filter_node = self._filter_signal
        return self

    def then_filter(self, filter_signal: SignalBase) -> Self:
        """Do further filter based on a given signal.

        Builds the clock signal filter, and then create a builder that performs cascade filtering.
        """
        signal_clock = self.build_clock_filter()
        ret = SignalFilterBuilder(filter_signal, signal_clock)
        if filter_signal.get_rust_type_name() == "bool":
            ret.filter_true()
        return ret

    def build_clock_filter(self) -> SignalBase:
        """Build filter based on the input filter signal's companion clock signal.

        Only each input signal from original data has corresponding clock signal can build clock filter."""
        if self._filter_node is None:
            raise ValueError("Not ready to build: no filter node")
        elif self._clock_signal:
            from ..processors import LevelTriggeredLatch
            return LevelTriggeredLatch(
                data=self._clock_signal,
                control=self._filter_node
            )
        else:
            raise ValueError("Input filter signal doesn't have a companion clock signal")

    def build_value_filter(self) -> SignalBase:
        if self._filter_node is None:
            raise ValueError("Not ready to build: no filter node")
        else:
            from ..processors import LevelTriggeredLatch
            return LevelTriggeredLatch(
                data=self._filter_signal,
                control=self._filter_node
            )
