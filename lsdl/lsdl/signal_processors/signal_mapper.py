from ..componet_base import BuiltinProcessorComponentBase


class SignalMapper(BuiltinProcessorComponentBase):
    def __init__(self, bind_var: str, lambda_src: str, upstream):
        lambda_decl = "|{bind_var}:&{bind_type}| {lambda_src}".format(
            bind_var = bind_var,
            bind_type = (upstream.get_rust_type_name()
                         if type(upstream) != list
                         else "(" + ",".join([e.get_rust_type_name() for e in upstream]) + ")"),
            lambda_src = lambda_src,
        )
        node_decl = "SignalMapper::new({lambda_decl})".format(
            lambda_decl = lambda_decl
        )
        super().__init__(
            name = "SignalMapper",
            node_decl = node_decl,
            upstreams = [upstream]
        )
