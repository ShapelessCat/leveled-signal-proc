from lsdl.componet_base import BuiltinComponentBase
from lsdl.signal import LeveledSignalBase

class StateMachineBuilder(object):
    def __init__(self, clock: LeveledSignalBase, data: LeveledSignalBase):
        self._clock = clock
        self._data = data
        self._transition_fn = '|_,_|()'
        self._scope_signal = None
        self._init_state = "Default::default()"
    def init_state(self, init_state):
        self._init_state = init_state
        return self
    def transition_fn(self, fn):
        self._transition_fn = fn
        return self
    def scoped(self, scope_signal: LeveledSignalBase):
        self._scope_signal = scope_signal
        return self
    def build(self): 
        if self._scope_signal is None:
            return StateMachine(
                clock = self._clock, 
                data = self._data, 
                transition_fn = self.transition_fn
            )
        else:
            actual_transition_fn = f"""{{
                let mut inner_fn = {self._transition_fn};
                move |&(last_scope, mut last_state), &(this_scope, ref this_input)|{{
                    if last_scope != this_scope {{
                        last_state = {self._init_state};
                    }}
                    (this_scope, (inner_fn)(&last_state, this_input))
                }}
            }}"""
            state_machine = StateMachine(
                clock = self._clock,
                data = [self._scope_signal, self._data],
                transition_fn = actual_transition_fn,
                init_state = f"(Default::default(), {self._init_state})",
            )
            return state_machine.map(bind_var = "&(_, s)", lambda_src = "s")

class StateMachine(BuiltinComponentBase):
    def __init__(self, clock:LeveledSignalBase, data:LeveledSignalBase, **kwargs):
        if 'transition_fn' in kwargs: 
            transition_fn = kwargs['transition_fn']
        else:
            raise "Currently only support transition_fn"
        init_state = kwargs.get("init_state", "Default::default()")
        node_decl = f"StateMachine::new({init_state}, {transition_fn})"
        super().__init__(
            name = "StateMachine",
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [clock, data]
        )