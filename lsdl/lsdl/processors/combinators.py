from enum import StrEnum
from typing import Callable, Optional

from ..lsp_model.core import SignalBase
from ..rust_code import RustCode


def make_tuple(*args: SignalBase) -> SignalBase:
    """Make a tuple from multiple input signals."""
    from . import SignalMapper

    return SignalMapper(
        bind_var="s", lambda_src="s.clone()", upstream=list(args)
    ).annotate_type(f'({",".join([arg.get_rust_type_name() for arg in args])})')


class FoldableOperation(StrEnum):
    SUM = "sum"
    MIN = "min"
    MAX = "max"
    AND = "and"
    OR = "or"


def time_domain_fold(
    fold_op: FoldableOperation,
) -> Callable[
    [SignalBase, Optional[SignalBase], Optional[SignalBase], Optional[RustCode]],
    SignalBase,
]:
    def inner(
        data: SignalBase,
        clock: Optional[SignalBase] = None,
        scope: Optional[SignalBase] = None,
        init_state: Optional[RustCode] = None,
    ) -> SignalBase:
        if clock is None:
            clock = data
        from . import StateMachineBuilder

        data_type = data.get_rust_type_name()
        lambda_param = f"s: &{data_type}, d: &{data_type}"

        match fold_op:
            case FoldableOperation.SUM:
                fold_method = f"|{lambda_param}| s.clone() + d.clone()"
                init_state = init_state or f"{data_type}::default()"
            case FoldableOperation.MIN:
                fold_method = f"|{lambda_param}| s.clone().min(d.clone())"
                init_state = init_state or f"{data_type}::MAX"
            case FoldableOperation.MAX:
                fold_method = f"|{lambda_param}| s.clone().max(d.clone())"
                init_state = init_state or f"{data_type}::MIN"
            case FoldableOperation.AND:
                fold_method = f"|{lambda_param}| *s && *d"
                init_state = init_state or "true"
            case FoldableOperation.OR:
                fold_method = f"|{lambda_param}| *s || *d"
                init_state = init_state or "false"

        builder = StateMachineBuilder(clock=clock, data=data)

        if init_state is not None:
            builder.init_state(init_state)

        builder.transition_fn(fold_method)

        if scope is not None:
            builder.scoped(scope)

        return builder.build().annotate_type(data.get_rust_type_name())

    return inner
