
import json
from time import sleep
from typing import Any, Optional, Self, final

from lsdl import RustCode

from lsdl.lsp_model.core import SignalBase
from lsdl.lsp_model.schema import Bool, CStyleEnum, DateTime, Float, Integer, LspEnumBase, MappedInputMember, SignalDataTypeBase, String, Vector, named
from lsdl.rust_code import INPUT_SIGNAL_BAG

_defined_nested_schema: Optional["GeneralInputSchemaBase"] = None


@final
class Object(SignalDataTypeBase):
    def __init__(self, name: RustCode):
        super().__init__(rust_type=name)


class GeneralInputSchemaBase(SignalBase):
    PROPERTIES_KEY = "properties"
    DEFS = "$defs"

    def __init__(self, type_name: RustCode = INPUT_SIGNAL_BAG):
        global _defined_nested_schema
        self.type_name = type_name
        # If treating `InputSchemaBase` itself as a signal/clock, its type should be `u64`.
        # Actually, lsp-codegen will always automatically insert a `_clock: u64` field
        # to the codegen result struct of this class, and the generated struct name should
        # be the value of `self.type_name`.
        super().__init__("u64")
        if "_timestamp_key" not in self.__dir__():
            self._timestamp_key = "timestamp"
        self._top_level_member_names = []
        for item_name in self.__dir__():
            if item_name not in ["_signal_data_type", "_signal_data_type"]:
                original_item = self.__getattribute__(item_name)
                if isinstance(original_item, SignalDataTypeBase | MappedInputMember):
                    GeneralInputSchemaBase._normalize(self, item_name, original_item)
                    self._top_level_member_names.append(item_name)
        _defined_nested_schema = self
    
    @staticmethod
    def _normalize(parent: Self | MappedInputMember, input_key: str, node: SignalDataTypeBase | MappedInputMember) -> None:
        match node:
            case SignalDataTypeBase():
                item = MappedInputMember(input_key, tpe=node)
                item.name = input_key
                parent.__setattr__(input_key, item)
                existing_attributes = node.__dir__()
                for child_item_name in existing_attributes:
                    if child_item_name not in ["_signal_data_type", "signal_data_type", "_parent"]:
                        original_child_item = node.__getattribute__(child_item_name)
                        if isinstance(original_child_item, SignalDataTypeBase | MappedInputMember):
                            delattr(node, child_item_name)
                            GeneralInputSchemaBase._normalize(parent=item, input_key=child_item_name, node=original_child_item)
            case MappedInputMember():
                node.name = input_key
                existing_attributes = node.__dir__()
                for child_item_name in existing_attributes:
                    if child_item_name not in ["_signal_data_type", "signal_data_type"]:
                        original_child_item = node.__getattribute__(child_item_name)
                        if isinstance(original_child_item, SignalDataTypeBase | MappedInputMember):
                            GeneralInputSchemaBase._normalize(parent=node, input_key=child_item_name, node=original_child_item)
    
    def to_patch_schema(self) -> dict:
        return self.to_dict()

    def to_schema(self) -> dict:
        ret: dict = {
            "$schema": "https://json-schema.org/draft-07/schema",
            "$comment": "InputSignalBag schema",
            "title": self.type_name,
            "type": "object",
            "patch_timestamp_key": self._timestamp_key,
            GeneralInputSchemaBase.PROPERTIES_KEY: {
                "_clock": {
                    "$ref": "#/$def/_clock"
                },
            },
            "required": ["_clock"],
            "$defs": {
                "_clock": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 18446744073709551615,
                    "input_key": "_clock",
                    "debug_info": self.debug_info.to_dict(),
                },
            },
        }

        properties = ret[GeneralInputSchemaBase.PROPERTIES_KEY]

        for name in self._top_level_member_names:
            member: MappedInputMember = getattr(self, name)
            properties[member.name] = { "$ref": f"#/$def/{member.name}" }
            ret["required"].append(member.name)
            if not isinstance(member.signal_data_type, Object):
                clock_name = member.clock().name
                properties[clock_name] =  { "$ref": f"#/$def/{clock_name}" }
                ret["required"].append(clock_name)
            GeneralInputSchemaBase._x_process_mapped_input_member(member, name, ret)
        return ret

    @staticmethod
    def _x_process_mapped_input_member(member: MappedInputMember, name: str, ret: dict[str, Any]):
        defs = ret[GeneralInputSchemaBase.DEFS]

        field_type_info = get_json_schema_type(member)
        defs[name] = field_type_info | {
            "input_key": member.get_input_key(),
            "debug_info": member.debug_info.to_dict(),
        }
        if isinstance(enum := member.signal_data_type, CStyleEnum):
            defs[name][
                "enum_variants"
            ] = enum.str_enum_type.variants_info()
        if member.reset_expr is not None:
            defs[name]["signal_behavior"] = {
                "name": "Reset",
                "default_expr": member.reset_expr,
            }
        
        if not isinstance(member.signal_data_type, Object):
            clock_name = member.clock().name
            defs[clock_name] = {
                "type": "integer",
                "minimum": 0,
                "maximum": 18446744073709551615,
                "input_key": member.get_input_key(),
                "debug_info": member.debug_info.to_dict(),
            }
        else:
            defs[name]["required"] = []
            required = defs[name]["required"]

            defs[name][GeneralInputSchemaBase.PROPERTIES_KEY] = {}
            inner_property = defs[name][GeneralInputSchemaBase.PROPERTIES_KEY]

            for inner_name in member.__dict__:
                inner_member = member.__getattribute__(inner_name)
                if isinstance(inner_member, MappedInputMember):
                    inner_property[inner_name] = { "$ref": f"#/$defs/{inner_name}" }
                    required.append(inner_name)
                    clock_name = inner_member.clock().name
                    if not isinstance(inner_member.signal_data_type, Object):
                        inner_property[clock_name] = { "$ref": f"#/$defs/{clock_name}" }
                        required.append(clock_name)
                    GeneralInputSchemaBase._x_process_mapped_input_member(inner_member, inner_name, ret)


    def to_dict(self) -> dict:
        ret: dict = {
            "$schema": "https://json-schema.org/draft-07/schema",
            "$comment": "InputSignalBagPatch schema",
            "title": f"{self.type_name}Patch",
            "type": "object",
            "patch_timestamp_key": self._timestamp_key,
            GeneralInputSchemaBase.PROPERTIES_KEY: {},
            "$defs": {}
        }

        properties = ret[GeneralInputSchemaBase.PROPERTIES_KEY]

        for name in self._top_level_member_names:
            member: MappedInputMember = getattr(self, name)
            properties[member.name] = { "$ref": f"#/$def/{member.name}" }
            GeneralInputSchemaBase._process_mapped_input_member(member, name, ret)
        return ret

    @staticmethod
    def _process_mapped_input_member(member: MappedInputMember, name: str, ret: dict[str, Any]):
        defs = ret[GeneralInputSchemaBase.DEFS]

        defs[name] = get_json_schema_type(member) | {
            "clock_companion": member.clock().name,
            "input_key": member.get_input_key(),
            "debug_info": member.debug_info.to_dict(),
        }
        if isinstance(enum := member.signal_data_type, CStyleEnum):
            defs[name][
                "enum_variants"
            ] = enum.str_enum_type.variants_info()
        if member.reset_expr is not None:
            defs[name]["signal_behavior"] = {
                "name": "Reset",
                "default_expr": member.reset_expr,
            }
        
        if isinstance(member.signal_data_type, Object):
            defs[name][GeneralInputSchemaBase.PROPERTIES_KEY] = {}
            inner_property = defs[name][GeneralInputSchemaBase.PROPERTIES_KEY]
            for inner_name in member.__dict__:
                inner_member = member.__getattribute__(inner_name)
                if isinstance(inner_member, MappedInputMember):
                    inner_property[inner_name] = { "$ref": f"#/$defs/{inner_name}" }
                    # inner_member.name = inner_name
                    GeneralInputSchemaBase._process_mapped_input_member(inner_member, inner_name, ret)


    def get_description(self):
        return {"type": "InputSignal", "id": "_clock"}
    
    @staticmethod
    def dfs(node: MappedInputMember, acc: dict[str, dict[str, Any]]):
        pass


def get_json_schema_type(member: MappedInputMember) -> dict[str, Any]:
    match member.signal_data_type:
        case Object():
            type_info = { "type": "object" }
        case String():
            type_info = { "type": "string" }
        case Integer():
            # TODO: Not all types below can be generated by typify exactly.
            type_info = { "type": "integer" }
            match member.get_rust_type_name():
                case "u8":
                    type_info |= {
                        "minimum": 0,
                        "maximum": 255
                    }
                case "i8":
                    type_info |= {
                        "minimum": -128,
                        "maximum": 127
                    }
                case "u16":
                    type_info |= {
                        "minimum": 0,
                        "maximum": 65535
                    }
                case "i16":
                    type_info |= {
                        "minimum": -32768,
                        "maximum": 32767
                    }
                case "u32":
                    type_info |= {
                        "minimum": 0,
                        "maximum": 4294967295
                    }
                case "i32":
                    type_info |= {
                        "minimum": -2147483648,
                        "maximum": 2147483647
                    }
                case "u64":
                    type_info |= {
                        "minimum": 0,
                        "maximum": 18446744073709551615
                    }
                case "i64":
                    type_info |= {
                        "minimum": -9223372036854775808,
                        "maximum": 9223372036854775807
                    }
                case "u128":
                    type_info |= {
                        "minimum": 0,
                        "maximum": 340282366920938463463374607431768211455
                    }
                case "i128":
                    type_info |= {
                        "minimum": -170141183460469231731687303715884105728,
                        "maximum": 170141183460469231731687303715884105727
                    }
        case Float():
            # TODO: It seems JSON Schema doesn't support distinguish `f64` and `f32`
            type_info = { "type": "number" }
            match member.get_rust_type_name():
                case "f32":
                    type_info |= {
                        "minimum": -3.4028235e+38,
                        "maximum": 3.4028235e+38
                    }
                case "f64":  # number is default to `f64`
                    pass
        case Bool():
            type_info = { "type": "bool" }
        case Vector():
            type_info = { "type": "array" }
        case CStyleEnum():
            type_info = { "type": "string" }
        case DateTime():
            type_info = {
                "type": "string",
                "format": "date-time"
            }
        
    return type_info


def get_nested_schema():
    return _defined_nested_schema


## TEST
@final
class Currency(LspEnumBase):
    Unknown = "UNKNOWN"
    Cny = "CNY"
    Euro = "EURO"
    Usd = "USD"


class NestableInputSignal(GeneralInputSchemaBase):
    _timestamp_key = "timestamp"

    fundamental = Object("Fundamental")

    fundamental.event_name = String()
    fundamental.event_category = String()

    fundamental.inner = Object("Inner")
    fundamental.inner.test = Float()

    encoded_fps = Float()
    inferred_rendered_fps = Float()

    currency = CStyleEnum(Currency)

    i8_int = Integer(signed=True, width=8)
    u8_int = Integer(signed=False, width=8)

    i16_int = Integer(signed=True, width=16)
    u16_int = Integer(signed=False, width=16)

    i32_int = Integer()
    u32_int = Integer(signed=False, width=32)

    i64_int = Integer(signed=True, width=64)
    u64_int = Integer(signed=False, width=64)

    i128_int = Integer(signed=True, width=128)
    u128_int = Integer(signed=False, width=128)

    f32_num = Float(width=32)
    f64_num = Float()

    data_time_0 = DateTime()

#     fundamental = named("fundamental", Object("Fundamental"))

#     fundamental.event_name = named("event_name")
#     fundamental.event_category = named("event_category")

#     fundamental.inner = named("inner", Object("Inner"))
#     fundamental.inner.test = named("test", Float())

#     encoded_fps = named("encoded_fps", Float())
#     inferred_rendered_fps = named("inferred_rendered_fps", Float())

#     currency = named("currency", CStyleEnum(Currency))

#     i8_int = named("i8_int", Integer(signed=True, width=8))
#     u8_int = named("u8_int", Integer(signed=False, width=8))

#     i16_int = named("i16_int", Integer(signed=True, width=16))
#     u16_int = named("u16_int", Integer(signed=False, width=16))

#     i32_int = named("i32_int", Integer())
#     u32_int = named("u32_int", Integer(signed=False, width=32))

#     i64_int = named("i64_int", Integer(signed=True, width=64))
#     u64_int = named("u64_int", Integer(signed=False, width=64))

#     i128_int = named("i64_int", Integer(signed=True, width=128))
#     u128_int = named("u64_int", Integer(signed=False, width=128))

#     f32_num = named("f32_num", Float(width=32))
#     f64_num = named("f64_num", Float())

#     data_time_0 = named("date_time_0", DateTime())


if __name__ == "__main__":
    nestable_input_signal = NestableInputSignal()

    schema = nestable_input_signal.to_schema()
    print(json.dumps(schema, indent=4), end='\n\n')

    # # repr = nestable_input_signal.to_dict()
    # patch_schema = nestable_input_signal.to_patch_schema()
    # print(json.dumps(patch_schema, indent=4), end='\n\n')
