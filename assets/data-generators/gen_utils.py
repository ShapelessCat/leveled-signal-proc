import logging
import random
from datetime import datetime, timedelta
from typing import Callable

logging.basicConfig(level=logging.INFO)


def timestamp_gen(
    initial_timestamp: datetime = datetime.now(),
) -> Callable[[timedelta], str]:
    return (
        lambda delta: f"{(initial_timestamp + delta).isoformat(sep=' ', timespec='milliseconds')} UTC"
    )


def random_bool():
    return bool(random.getrandbits(1))


RATE_OF_KEEPING_LAST_TIMESTAMP = 0.01


def generate_timestamps(count: int):
    timestamp_of = timestamp_gen()
    time_delta = timedelta(seconds=0)
    for _ in range(count):
        is_simultaneous = random.random() <= RATE_OF_KEEPING_LAST_TIMESTAMP
        if not is_simultaneous:
            time_delta += timedelta(seconds=random.randint(10, 1000) / 10.0)

        t = timestamp_of(time_delta)
        if is_simultaneous:
            logging.info("No timestamp change for %s", t)
        yield t
