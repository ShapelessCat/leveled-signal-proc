from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class StateMachine(BuiltinComponentBase):
    def __init__(self, clock:LeveledSignalBase, data:LeveledSignalBase, **kwargs):
        if 'transition_fn' in kwargs: 
            node_decl = "StateMachine::new(0, {transition_fn})".format(
                transition_fn = kwargs['transition_fn']
            )
        else:
            raise "Currently only support transition_fn"
        super().__init__(
            name = "StateMachine",
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [clock, data]
        )
        self._output_type = "i32"