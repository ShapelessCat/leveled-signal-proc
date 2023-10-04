from datetime import datetime


def timestamp(delta):
    return (datetime.now() + delta).strftime('%Y-%m-%d %H:%M:%S.%f') + ' UTC'
