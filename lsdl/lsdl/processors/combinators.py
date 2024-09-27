from typing import Optional

from ..lsp_model.core import SignalBase


def make_tuple(*args: SignalBase) -> SignalBase:
    """Make a tuple from multiple input signals."""
    from . import SignalMapper

    return SignalMapper(
        bind_var="s", lambda_src="s.clone()", upstream=list(args)
    ).annotate_type(f'({",".join([arg.get_rust_type_name() for arg in args])})')


def time_domain_fold(
    data: SignalBase,
    clock: Optional[SignalBase] = None,
    scope: Optional[SignalBase] = None,
    fold_method="sum",
    init_state=None,
):
    if clock is None:
        clock = data
    from . import StateMachineBuilder

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
    elif fold_method == "and":
        fold_method = f"|{lambda_param}| *s && *d"
        init_state = "true" if init_state is None else init_state
    elif fold_method == "or":
        fold_method = f"|{lambda_param}| *s || *d"
        init_state = "false" if init_state is None else init_state
    builder = StateMachineBuilder(clock=clock, data=data)

    if init_state is not None:
        builder.init_state(init_state)

    builder.transition_fn(fold_method)

    if scope is not None:
        builder.scoped(scope)

    return builder.build().annotate_type(data.get_rust_type_name())
