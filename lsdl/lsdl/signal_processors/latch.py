from ..componet_base import BuiltinProcessorComponentBase
from ..signal import SignalBase


class Latch(BuiltinProcessorComponentBase):
    def __init__(self,
                 control: SignalBase,
                 data: SignalBase,
                 forget_duration: int | str = -1,
                 **kwargs):
        rust_processor_name = self.__class__.__name__
        from ..modules import normalize_duration
        forget_duration = normalize_duration(forget_duration)
        dt = data.get_rust_type_name()
        if forget_duration < 0:
            node_decl = f"{rust_processor_name}::<{dt}>::default()"
        else:
            default = f"<{dt} as Default>::default()"
            node_decl = f"""
                {rust_processor_name}::with_forget_behavior(
                    {default}, {default}, {forget_duration}
                )
            """
        super().__init__(
            name=rust_processor_name,
            node_decl=node_decl,
            upstreams=[control, data]
        )
        key4type = "output_type"
        if key4type in kwargs:
            self.annotate_type(kwargs[key4type])
        else:
            self.annotate_type(data.get_rust_type_name())


class EdgeTriggeredLatch(BuiltinProcessorComponentBase):
    def __init__(self,
                 control: SignalBase,
                 data: SignalBase,
                 forget_duration: int | str = -1,
                 **kwargs):
        rust_processor_name = self.__class__.__name__
        from ..modules import normalize_duration
        forget_duration = normalize_duration(forget_duration)
        dt = data.get_rust_type_name()
        if forget_duration < 0:
            ct = control.get_rust_type_name()
            node_decl = f"{rust_processor_name}::<{ct}, {dt}>::default()"
        else:
            default = f"<{dt} as Default>::default()"
            node_decl = f"""
                {rust_processor_name}::with_forget_behavior(
                    {default}, {default}, {forget_duration}
                )
            """
        super().__init__(
            name=rust_processor_name,
            node_decl=node_decl,
            upstreams=[control, data]
        )
        key4type = "output_type"
        if key4type in kwargs:
            self.annotate_type(kwargs[key4type])
        else:
            self.annotate_type(data.get_rust_type_name())
