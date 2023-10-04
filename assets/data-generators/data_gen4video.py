#!/usr/bin/env python

import logging
import random
import sys
from datetime import timedelta
from pathlib import Path

from gen_utils import timestamp

logging.basicConfig(level=logging.INFO)

SEEK_START = 'seek start'
SEEK_END = 'seek end'

candidate_data = {
    'ev': [SEEK_START, SEEK_END],
    'CDN': ['c1', 'c2', 'c3'],
    'PlayerState': ['playing', 'buffering', 'pause'],
    'placeholder': ['p0', 'p1'],
    'BitRate': []
}

candidate_data_keys = list(candidate_data.keys())

seek_value = SEEK_END


def random_event_data():
    key = random.choice(candidate_data_keys)
    choices = candidate_data[key]
    if key == 'ev':
        global seek_value
        seek_value = SEEK_START if seek_value == SEEK_END else SEEK_END
        value = f'"{seek_value}"'
    elif choices:
        value = f'"{random.choice(choices)}"'
    else:
        value = random.randint(1, 5) * 1000

    return f'"{key}": {value}'


if __name__ == '__main__':
    output_file = sys.stdout
    if len(sys.argv) < 2 or not sys.argv[1].isdigit():
        logging.error('This script only accept one integer argument, which represent the required number of entries.')
        sys.exit(1)
    else:
        required_count = int(sys.argv[1])
        default_output_path = Path(__file__).parent.parent / 'data' / 'video-metrics-demo-input.jsonl'
        output_path = sys.argv[2] if len(sys.argv) >= 3 else default_output_path
        if output_path != "-":
            output_file = open(output_path, "w")

    template = '''{{"timestamp": "{}", "sessionId": "{}", {}}}'''

    session_id = 0
    time_delta = timedelta(seconds=0)

    for i in range(required_count):
        should_switch_session = i % random.randint(1, 20) == 0 and bool(random.getrandbits(1))
        if should_switch_session:
            session_id += 1
        time_delta += timedelta(seconds=random.randint(10, 1000) / 10.0)
        recbuf = template.format(timestamp(time_delta), f'SSID_{session_id}', random_event_data())
        print(recbuf, file = output_file)