from inspect import getframeinfo, stack
from pathlib import Path
from typing import Any, final


@final
class DebugInfo:
    def __init__(self):
        package_root = Path(__file__)
        while package_root.name != __package__:
            package_root = package_root.parent
        stack_info = stack()
        self._file = "<unknown>"
        self._line = -1
        for f in stack_info:
            traceback = getframeinfo(f.frame)
            file_path = Path(traceback.filename)
            is_in_package = False
            while not is_in_package and file_path.parent != file_path:
                is_in_package = file_path.parent == package_root
                file_path = file_path.parent
            if not is_in_package:
                self._file = traceback.filename
                self._line = traceback.lineno
                break

    def to_dict(self) -> dict[str, Any]:
        return {
            "file": self._file,
            "line": self._line,
        }
