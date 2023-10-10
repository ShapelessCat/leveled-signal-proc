from inspect import stack, getframeinfo
from pathlib import Path


class DebugInfo:
    def __init__(self):
        package_root = Path(__file__)
        while package_root.name != __package__:
            package_root = package_root.parent
        stack_info = stack()
        self._file = "<unknown>"
        self._line = -1
        for f in stack_info:
            f = getframeinfo(f.frame)
            file_path = Path(f.filename)
            is_in_package = False
            while not is_in_package and file_path.parent != file_path:
                is_in_package = file_path.parent == package_root
                file_path = file_path.parent
            if not is_in_package:
                self._file = f.filename
                self._line = f.lineno
                break

    def to_dict(self):
        return {
            "file": self._file,
            "line": self._line,
        }
