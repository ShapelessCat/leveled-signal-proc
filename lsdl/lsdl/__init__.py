import configparser
import json
import os

from .config import measurement_config, processing_config

__config = configparser.ConfigParser()
__current_file_path = os.path.dirname(os.path.abspath(__file__))
__config.read(f"{__current_file_path}/rust_keywords.ini")
__strict_and_reserved_rust_keywords = {*__config['strict'].values(), *__config['reserved'].values()}


# TODO: We should allow all legal Rust identifiers.
def validate_rust_identifier(identifier: str) -> None:
    """Check if an identifier is a legal Rust identifier.

    For implementation simplicity, only a C-style identifier that is not a Rust strict/reserved
    keyword is allowed.

    CAUTION:
    Current check is easy to implement, but it is also too strict. We should allow all legal Rust
    identifier.
    """
    import re
    regex = '^[A-Za-z_][A-Za-z0-9_]*$'
    if not re.match(regex, identifier) or identifier in __strict_and_reserved_rust_keywords:
        raise Exception(f"{identifier} is not a simple and legal Rust identifier!")


def _get_json_ir(pretty_print=False) -> str:
    from .componet_base import get_components
    from .schema import get_schema
    ret_obj = {
        "schema": get_schema().to_dict(),
        "nodes": [c.to_dict() for c in get_components()],
        "measurement_policy": measurement_config().to_dict(),
        "processing_policy": processing_config().to_dict(),
    }
    return json.dumps(ret_obj, indent=4 if pretty_print else None)


def print_ir_to_stdout():
    print(_get_json_ir(True))
