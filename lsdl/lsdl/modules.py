from lsdl.signal import LeveledSignalBase
from lsdl.schema import MappedInputType
from lsdl.signal_processors import SignalMapper, Latch
from lsdl.const import Const
import re

def _normalize_duration(duration) -> int:
    if type(duration) == str:
        value_str = re.search(r"\d+", duration).group()[0]
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
    
"""
def event_filter(event_signal: MappedInputType, **kwargs) -> LeveledSignalBase:
    from lsdl.signal_processors import SignalMapper, Latch
    if 'filter_input' not in kwargs:
        kwargs['filter_input'] = event_signal
    if 'event_clock' not in kwargs:
        kwargs['event_clock'] = event_signal.clock()
    if 'event_value' in kwargs:
        if type(kwargs['event_value']) == list:
            filter_node = (kwargs['filter_input'] == kwargs['event_value'][0])
            for value in kwargs['event_value'][1:]:
                filter_node = filter_node | (kwargs['filter_input'] == value)
        else:
            filter_node = (kwargs['filter_input'] == kwargs['event_value'])
    elif 'filter_lambda' in kwargs:
        filter_node = SignalMapper(
            bind_var = kwargs['bind_var'],
            upstream = kwargs['filter_input'],
            lambda_src = kwargs['filter_lambda']
        )
    elif 'filter_node' in kwargs:
        filter_node = kwargs['filter_node']
    else:
        raise "Unsupported event filter type"
    if kwargs.get('output') == "value":
        return Latch(
            control = filter_node,
            data = kwargs['filter_input'] 
        )
    else:
        return Latch(
            control = filter_node,
            data = kwargs['event_clock']
        )"""
