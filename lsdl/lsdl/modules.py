
from lsdl.signal import LeveledSignalBase

def has_been_true(input: LeveledSignalBase, duration: int = -1) -> LeveledSignalBase:
    from lsdl.signal_processors import Latch
    from lsdl.const import Const
    return Latch(
            data = Const(True),
            control = input,
            forget_duration = duration
        )
