from typing import Optional, final

from ..lsp_model.component_base import BuiltinProcessorComponentBase
from ..lsp_model.core import SignalBase
from ..rust_code import RUST_DEFAULT_VALUE, RustCode


@final
class StateMachineBuilder:
    def __init__(
        self, clock: SignalBase | list[SignalBase], data: SignalBase | list[SignalBase]
    ):
        self._clock = clock
        self._data = data
        self._transition_fn = "|_, _| ()"
        self._scope_signal: Optional[SignalBase] = None
        self._init_state = RUST_DEFAULT_VALUE

    def init_state(self, init_state: RustCode):
        self._init_state = init_state
        return self

    def transition_fn(self, fn: RustCode):
        self._transition_fn = fn
        return self

    def scoped(self, scope_signal: SignalBase):
        self._scope_signal = scope_signal
        return self

    def build(self):
        if self._scope_signal is None:
            return StateMachine(
                clock=self._clock, data=self._data, transition_fn=self._transition_fn
            )
        else:
            # When a type in `self._transition_fn` can't be inferred, it seems
            # sometime the compiler doesn't know the exact reason, and it panics
            # with the error code [E0521] "borrowed data escapes outside of
            # closure", combined with a message "`inner_fn` declared here,
            # outside the closure body`. When this happens, don't try to move
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
                clock=[self._scope_signal, self._clock],
                data=[self._scope_signal, self._clock, self._data],
                transition_fn=actual_transition_fn,
                init_state=f"({RUST_DEFAULT_VALUE}, {RUST_DEFAULT_VALUE}, {self._init_state})",
            )
            return state_machine.map(bind_var="&(_, _, s)", lambda_src="s")


@final
class StateMachine(BuiltinProcessorComponentBase):
    def __init__(
        self,
        clock: SignalBase | list[SignalBase] | list[SignalBase | list[SignalBase]],
        data: SignalBase | list[SignalBase] | list[SignalBase | list[SignalBase]],
        **kwargs,
    ):
        if "transition_fn" in kwargs:
            transition_fn = kwargs["transition_fn"]
        else:
            raise ValueError("Currently only support transition_fn")
        rust_processor_name = self.__class__.__name__
        init_state = kwargs.get("init_state", RUST_DEFAULT_VALUE)
        super().__init__(
            name=rust_processor_name,
            node_decl=f"{rust_processor_name}::new({init_state}, {transition_fn})",
            upstreams=[clock, data],
        )
