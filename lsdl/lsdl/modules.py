
from lsdl.signal import LeveledSignalBase
from lsdl.schema import MappedInputType

def has_been_true(input: LeveledSignalBase, duration: int = -1) -> LeveledSignalBase:
    from lsdl.signal_processors import Latch
    from lsdl.const import Const
    return Latch(
            data = Const(True),
            control = input,
            forget_duration = duration
        )

def event_filter(event_signal: MappedInputType, **kwargs) -> LeveledSignalBase:
    from lsdl.signal_processors import SignalMapper, Latch
    if 'filter_input' not in kwargs:
        kwargs['filter_input'] = event_signal
    if 'event_clock' not in kwargs:
        kwargs['event_clock'] = event_signal.clock()
    if 'event_value' in kwargs:
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
    return Latch(
        control = filter_node,
        data = kwargs['event_clock']
    )
