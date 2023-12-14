from lsdl.measurement import MeasurementBase
from ..componet_base import BuiltinMeasurementComponentBase
from ..signal import SignalBase


class Peek(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rule_component_name = self.__class__.__name__
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type(input_signal.get_rust_type_name())


class PeekTimestamp(BuiltinMeasurementComponentBase):
    def __init__(self, input_signal: SignalBase):
        rule_component_name = self.__class__.__name__
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::default()",
            upstreams=[input_signal]
        )
        self.annotate_type("u64")

class MeasureMapper(BuiltinMeasurementComponentBase):
    def __init__(self, bind_var: str, lambda_src: str, m: MeasurementBase):
        rule_component_name = self.__class__.__name__
        how = f"|{bind_var}| {lambda_src}"
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::new({how}, ???{m})",
            upstreams=[input_signal]
        )
        self.annotate_type("u64")



# class SignalMapper(BuiltinProcessorComponentBase):
#     def __init__(self, bind_var: str, lambda_src: str, upstream: SignalBase | list[SignalBase]):
#         bind_type = (upstream.get_rust_type_name()
#                      if not isinstance(upstream, list)
#                      else "(" + ",".join([e.get_rust_type_name() for e in upstream]) + ")")
#         lambda_decl = f"|{bind_var}:&{bind_type}| {lambda_src}"
#         rust_processor_name = self.__class__.__name__
#         super().__init__(
#             name=rust_processor_name,
#             node_decl=f"{rust_processor_name}::new({lambda_decl})",
#             upstreams=[upstream]
#         )
