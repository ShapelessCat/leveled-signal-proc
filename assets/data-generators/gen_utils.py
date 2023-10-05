from datetime import datetime
import random


def timestamp(delta):
    return (datetime.now() + delta).strftime('%Y-%m-%d %H:%M:%S.%f') + ' UTC'


def random_bool():
    return bool(random.getrandbits(1))
