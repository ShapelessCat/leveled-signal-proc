from curses.ascii import isdigit
import logging
import random
import sys
from datetime import datetime, timedelta
from pathlib import Path

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

def timestamp(delta):
    return (datetime.now() + delta).strftime('%Y-%m-%d %H:%M:%S.%f') + ' UTC'

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
    if len(sys.argv) != 2 or not sys.argv[1].isdigit():
        logging.error('This script only accept one integer argument, which represent the required number of entries.')
        sys.exit(1)
    else:
        required_count = int(sys.argv[1])

        result = []
        template = '''{{"timestamp": "{}", "sessionId": "{}", {}}}'''

        session_id = 0
        time_delta = timedelta(seconds = 0)

        for i in range(required_count):
            should_switch_session = i % random.randint(1, 20) == 0 and bool(random.getrandbits(1))
            if should_switch_session:
                session_id += 1
            time_delta += timedelta(seconds = random.randint(10, 1000) / 10.0)
            result.append(template.format(timestamp(time_delta), f'SSID_{session_id}', random_event_data())) 

        output_path = Path(__file__).parent.parent / 'data' / 'video-metrics-demo-input.jsonl'
        print('\n'.join(result), file=open(output_path, 'w'))
