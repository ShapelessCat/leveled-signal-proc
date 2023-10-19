from ..componet_base import BuiltinMeasurementComponentBase
from ..signal import SignalBase


class DiffSinceCurrentLevel(BuiltinMeasurementComponentBase):
    def __init__(self, control: SignalBase, data: SignalBase):
        rule_component_name = self.__class__.__name__
        super().__init__(
            name=rule_component_name,
            node_decl=f"{rule_component_name}::default()",
            upstreams=[control, data],
        )
        self.annotate_type(data.get_rust_type_name())
