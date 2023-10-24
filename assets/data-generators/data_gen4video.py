#!/usr/bin/env python3

import logging
import random
import sys
from pathlib import Path

from gen_utils import generate_timestamps, random_bool

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
        logging.error(
            'Only accept one integer argument that specifies the required number of entries.'
        )
        sys.exit(1)
    else:
        required_count = int(sys.argv[1])
        sample_data_home = Path(__file__).parent.parent / 'data'
        default_output_path = sample_data_home / 'video-metrics-demo-input.jsonl'
        output_path = sys.argv[2] if len(sys.argv) >= 3 else default_output_path
        if output_path != "-":
            output_file = open(output_path, 'w', encoding='utf-8')

    TEMPLATE = '''{{"timestamp": "{}", "sessionId": "{}", {}}}'''

    session_id = 0
    for i, t in enumerate(generate_timestamps(required_count)):
        should_switch_session = i % random.randint(1, 20) == 0 and random_bool()
        if should_switch_session:
            session_id += 1
        recbuf = TEMPLATE.format(t, f"SSID_{session_id}", random_event_data())
        print(recbuf, file=output_file)
