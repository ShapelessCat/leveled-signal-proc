from lsdl.componet_base import BuiltinComponentBase

class SignalMapper(BuiltinComponentBase):
    def __init__(self, bind_var: str, lambda_src: str, upstream):
        lambda_decl = "|{bind_var}:&{bind_type}| {lambda_src}".format(
            bind_var = bind_var, 
            bind_type = upstream.get_rust_type_name() if type(upstream) != list else "(" + ",".join([e.get_rust_type_name() for e in upstream]) + ")",
            lambda_src = lambda_src,
        )
        node_decl = "SignalMapper::new({lambda_decl})".format(
            lambda_decl = lambda_decl
        )
        super().__init__(
            name = "SignalMapper",
            is_measurement = False, 
            node_decl = node_decl, 
            upstreams = [upstream]
        )