from lsdl.signal import LeveledSignalBase
from lsdl.schema import MappedInputType
from lsdl.signal_processors import SignalMapper, Latch
from lsdl.const import Const
import re

def _normalize_duration(duration) -> int:
    if type(duration) == str:
        value_str = re.search(r"\d+", duration).group(0)
        value_unit = duration[len(value_str):]
        value = int(value_str)
        if value_unit == "s":
            duration = value * 1_000_000_000
        elif value_unit == "ms":
            duration = value * 1_000_000
        elif value_unit == "us":
            duration = value * 1_000
        elif duration == "ns":
            duration = value
        elif duration == "m":
            duration = value * 60_000_000_000
        elif duration == "h":
            duration = value * 3_600_000_000_000
    return duration

def has_been_true(input: LeveledSignalBase, duration = -1) -> LeveledSignalBase:
    return Latch(
            data = Const(True),
            control = input,
            forget_duration = _normalize_duration(duration)
        )

def make_tuple(*args) -> LeveledSignalBase:
    return SignalMapper(
        bind_var = "s",
        lambda_src = "s.clone()",
        upstream = list(args)
    ).annotate_type(f'({",".join([arg.get_rust_type_name() for arg in args])})')

class SignalFilterBuilder(object):
    def __init__(self, filter_signal: LeveledSignalBase, clock_signal: LeveledSignalBase = None):
        self._filter_signal = filter_signal
        self._clock_signal = clock_signal
        if isinstance(filter_signal, MappedInputType) and clock_signal is None:
            self._clock_signal = filter_signal.clock()
        self._filter_lambda = None
    def filter_fn(self, bind_var: str, lambda_body: str):
        self._filter_node = SignalMapper(
            bind_var = bind_var,
            upstream = self._filter_signal,
            lambda_src = lambda_body, 
        )
        return self
    def filter_values(self, *args):
        values = args
        self._filter_node = (self._filter_signal == values[0])
        for value in values[1:]:
            self._filter_node = self._filter_node | (self._filter_signal == value)
        return self
    def build_clock_filter(self) -> LeveledSignalBase:
        return Latch(
            data = self._clock_signal,
            control = self._filter_node
        )
    def build_value_filter(self) -> LeveledSignalBase:
        return Latch(
            data = self._filter_signal,
            control = self._filter_node
        )
    
class ScopeContext(object):
    def __init__(self, scope_level: LeveledSignalBase, epoch: LeveledSignalBase):
        self._scope = scope_level
        self._epoch = epoch
    def scoped(self, data: LeveledSignalBase, clock: LeveledSignalBase, default = None) -> LeveledSignalBase:
        from lsdl.signal_processors import EdgeTriggeredLatch, SignalMapper
        scope_starts = EdgeTriggeredLatch(control = self._scope, data = self._epoch)
        event_starts = EdgeTriggeredLatch(control = data, data = self._epoch)
        return SignalMapper(
            bind_var = "(sep, eep, signal)", 
            lambda_src = f"""if *sep <= *eep {{ signal.clone() }} else {{ 
                { "Default::default()" if default is None else str(default) }
            }}""", 
            upstream = [scope_starts, event_starts, data]
        ).annotate_type(data.get_rust_type_name())

def time_domain_fold(data: LeveledSignalBase, clock = None, scope = None, fold_method = "sum", init_state = None):
    if clock is None:
        clock = data
    from lsdl.signal_processors.state_machine import StateMachineBuilder
    data_type = data.get_rust_type_name()
    lambda_param = f"s: &{data_type}, d: &{data_type}"
    if fold_method == "sum":
        fold_method = f"|{lambda_param}| s.clone() + d.clone()"
        init_state = f"{data_type}::default()" if init_state is None else init_state
    elif fold_method == "min":
        fold_method = f"|{lambda_param}| s.clone().min(d.clone())"
        init_state = f"{data_type}::MAX" if init_state is None else init_state
    elif fold_method == "max":
        fold_method = f"|{lambda_param}| s.clone().max(d.clone())"
        init_state = f"{data_type}::MIN" if init_state is None else init_state
    builder = StateMachineBuilder(clock = clock, data = data)

    if init_state is not None:
        builder.init_state(init_state)

    builder.transition_fn(fold_method)

    if scope is not None:
        builder.scoped(scope)

    return builder.build().annotate_type(data.get_rust_type_name())