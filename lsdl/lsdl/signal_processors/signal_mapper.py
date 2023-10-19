from lsdl.signal import SignalBase
from ..componet_base import BuiltinProcessorComponentBase


class SignalMapper(BuiltinProcessorComponentBase):
    def __init__(self, bind_var: str, lambda_src: str, upstream: SignalBase | list[SignalBase]):
        bind_type = (upstream.get_rust_type_name()
                     if not isinstance(upstream, list)
                     else "(" + ",".join([e.get_rust_type_name() for e in upstream]) + ")")
        lambda_decl = f"|{bind_var}:&{bind_type}| {lambda_src}"
        rust_processor_name = self.__class__.__name__
        super().__init__(
            name=rust_processor_name,
            node_decl=f"{rust_processor_name}::new({lambda_decl})",
            upstreams=[upstream]
        )


def _build_signal_mapper(
    cond: SignalBase,
    then_branch: SignalBase,
    else_branch: SignalBase
) -> SignalBase:
    inner = SignalMapper(
        bind_var="(cond, then_expr, else_expr)",
        lambda_src="if *cond { then_expr.clone() } else { else_expr.clone() }",
        upstream=[cond, then_branch, else_branch]
    )
    else_type = else_branch.get_rust_type_name()
    then_type = then_branch.get_rust_type_name()
    if then_type == "_":
        then_type = else_type
    elif else_type == "_":
        else_type = then_type

    if then_type == else_type:
        inner.annotate_type(then_type)
    return inner


class If(SignalBase):
    """The `if...then...else` expression for a leveled signal."""
    def __init__(self,
                 cond_expr: SignalBase,
                 then_expr: SignalBase,
                 else_expr: SignalBase):
        super().__init__()
        self._inner = _build_signal_mapper(cond_expr, then_expr, else_expr)

    def get_id(self):
        return self._inner.get_id()

    def get_rust_type_name(self) -> str:
        return self._inner.get_rust_type_name()


class Cond(SignalBase):
    """The scheme `cond` style expression for a leveled signal."""
    def __init__(self,
                 first_branch: (SignalBase, SignalBase),
                 middle_branches: [(SignalBase, SignalBase)],
                 fallback_value: SignalBase):
        super().__init__()
        self._inner = _build_signal_mapper(*first_branch, fallback_value)
        while middle_branches:
            (cond, then_branch) = middle_branches.pop()
            self._inner = _build_signal_mapper(cond, then_branch, self._inner)

    def get_id(self):
        return self._inner.get_id()

    def get_rust_type_name(self) -> str:
        return self._inner.get_rust_type_name()
