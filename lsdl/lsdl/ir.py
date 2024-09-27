import json

from .config import measurement_config, processing_config


def _get_json_ir(pretty_print=False) -> str:
    from .lsp_model.component_base import get_components
    from .lsp_model.schema import get_schema

    ret_obj = {
        "schema": get_schema().to_dict(),
        "nodes": [c.to_dict() for c in get_components()],
        "measurement_policy": measurement_config().to_dict(),
        "processing_policy": processing_config().to_dict(),
    }
    return json.dumps(ret_obj, indent=4 if pretty_print else None)


def print_ir_to_stdout():
    print(_get_json_ir(True))
