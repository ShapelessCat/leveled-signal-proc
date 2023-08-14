import json
from typing import Any

from lsdl.config import measurement_config

def get_json_ir(pretty_print = False) -> str:
    from lsdl.componet_base import get_components
    from lsdl.schema import get_schema
    ret_obj = {
        "schema": get_schema().to_dict(),
        "nodes": [c.to_dict() for c in get_components()],
        "measurement_policy": measurement_config().to_dict(),
    }
    return json.dumps(ret_obj, indent = 4 if pretty_print else None)

def print_ir_to_stdout():
    print(get_json_ir(True))