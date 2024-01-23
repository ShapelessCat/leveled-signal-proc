import re


def normalize_duration(duration: int | str) -> int:
    if isinstance(duration, str):
        value_str = re.search(r"\d+", duration).group(0)
        value_unit = duration[len(value_str):]
        value = int(value_str)
        match value_unit:
            case 's':
                duration = value * 1_000_000_000
            case 'ms':
                duration = value * 1_000_000
            case 'us':
                duration = value * 1_000
            case 'ns':
                duration = value
            case 'm':
                duration = value * 60_000_000_000
            case 'h':
                duration = value * 3_600_000_000_000
            case _:
                raise ValueError(f"Unknown duration unit: {value_unit}")
    return duration
