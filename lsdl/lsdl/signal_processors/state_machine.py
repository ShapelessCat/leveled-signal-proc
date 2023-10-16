from ..componet_base import BuiltinProcessorComponentBase
from ..signal import LeveledSignalProcessingModelComponentBase


class StateMachineBuilder:
    def __init__(self, clock: LeveledSignalProcessingModelComponentBase, data: LeveledSignalProcessingModelComponentBase):
        self._clock = clock
        self._data = data
        self._transition_fn = '|_,_|()'
        self._scope_signal = None
        self._init_state = "Default::default()"

    def init_state(self, init_state):
        self._init_state = init_state
        return self

    def transition_fn(self, fn: str):
        self._transition_fn = fn
        return self

    def scoped(self, scope_signal: LeveledSignalProcessingModelComponentBase):
        self._scope_signal = scope_signal
        return self

    def build(self):
        if self._scope_signal is None:
            return StateMachine(
                clock = self._clock,
                data = self._data,
                transition_fn = self._transition_fn
            )
        else:
            # When a type in `self._transition_fn` can't be inferred, it seems
            # sometime the compiler doesn't know the exact reason, and it panics
            # with the error code [E0521] "borrowed data escapes outside of
            # closure", combined with a message "`inner_fn` declared here,
            # outside of the closure body`. When this happen, don't try to move
            # the `inner_fn` here, and we should add more type annotations to
            # this `self._transition_fn`.
            actual_transition_fn = f"""{{
                let inner_fn = {self._transition_fn};
                move |&(last_scope, last_clock, mut last_state),
                      &(this_scope, this_clock, ref this_input)|{{
                    if last_scope != this_scope {{
                        last_state = {self._init_state};
                    }}
                    if last_clock == this_clock {{
                        (this_scope, this_clock, last_state)
                    }}
                    else {{
                        (this_scope, this_clock, (inner_fn)(&last_state, this_input))
                    }}
                }}
            }}"""
            state_machine = StateMachine(
                clock = [self._scope_signal, self._clock],
                data = [self._scope_signal, self._clock, self._data],
                transition_fn = actual_transition_fn,
                init_state = f"(Default::default(), Default::default(), {self._init_state})",
            )
            return state_machine.map(bind_var = "&(_, _, s)", lambda_src = "s")


class StateMachine(BuiltinProcessorComponentBase):
    def __init__(self, clock: LeveledSignalProcessingModelComponentBase, data: LeveledSignalProcessingModelComponentBase, **kwargs):
        if 'transition_fn' in kwargs:
            transition_fn = kwargs['transition_fn']
        else:
            raise "Currently only support transition_fn"
        init_state = kwargs.get("init_state", "Default::default()")
        node_decl = f"StateMachine::new({init_state}, {transition_fn})"
        super().__init__(
            name = "StateMachine",
            node_decl = node_decl,
            upstreams = [clock, data]
        )
