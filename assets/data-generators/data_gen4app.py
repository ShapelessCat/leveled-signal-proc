#!/usr/bin/env python

import inspect
import json
import logging
import os
import random
import re
import sys
from abc import ABC, abstractmethod
from datetime import timedelta
from enum import Enum
from pathlib import Path
from typing import Type

from gen_utils import timestamp

logging.basicConfig(level=logging.INFO)


class Platform(Enum):
    MOB = 1
    WEB = 2

    def __str__(self):
        return f'{self.name.lower()}'


# TODO: Not in use. Can be used to make the generated data more like real data.
class FixedFrequency(ABC):
    FIXED_TIME_INTERVAL = 40000  # ms


class Event(ABC):
    PAGE_LOAD_TIME_THRESHOLD = 90000    # ms
    SCREEN_LOAD_TIME_THRESHOLD = 60000  # ms

    EVENT_CATEGORY_POOL = [f"ec{i}" for i in range(3)]

    @abstractmethod
    def __init__(self, event_name: str, platform: Platform):
        self.event_name = event_name
        self.platform = str(platform)

    def generate(self):
        basic_info = {
            'event_name': self.event_name,
            'event_category': random.choice(self.EVENT_CATEGORY_POOL),
            'platform': self.platform,
        }
        no_load_info = bool(random.getrandbits(1))
        load_threshold = (self.PAGE_LOAD_TIME_THRESHOLD
                          if self.platform is Platform.WEB
                          else self.SCREEN_LOAD_TIME_THRESHOLD)
        extra_info = ({}
                      if no_load_info
                      else self._random_time_interval('load_start', 'load_end', load_threshold))
        return basic_info | extra_info

    # This time interval can be valid or invalid
    @classmethod
    def _random_time_interval(cls, start_key: str, end_key: str, threshold: int) -> dict[str, str]:
        smaller = random.randint(0, 10)
        greater = random.randint(10, 1000)
        valid_order = bool(random.getrandbits(1))
        return {
            start_key: str(smaller if valid_order else greater),
            end_key: str((greater if valid_order else smaller) + random.choice([0, 10, 100, threshold]))
        }

    @classmethod
    def _camel_case2snake_case(cls, name: str) -> str:
        return re.sub(r'(?<!^)(?=[A-Z])', '_', name).lower()


class RandomNameEvent(Event):
    def __init__(self, platform: Platform):
        super().__init__('random_event_' + str(random.randint(1, 10)), platform)


class MobileOnlyEvent(Event, ABC):
    @abstractmethod
    def __init__(self):
        event_name = self._camel_case2snake_case(self.__class__.__name__)
        super().__init__(event_name, Platform.MOB)


class ConvivaPeriodicHeartbeat(MobileOnlyEvent, FixedFrequency):
    def __init__(self):
        super().__init__()


class ConvivaScreenView(MobileOnlyEvent):
    APP_STARTUP_TIME_THRESHOLD = 300000  # ms

    def __init__(self):
        super().__init__()

    def generate(self):
        basic_info = super().generate()
        no_previous_exist = bool(random.getrandbits(1))
        return (
                basic_info |
                self._random_time_interval('app_startup_start',
                                           'app_startup_end',
                                           self.APP_STARTUP_TIME_THRESHOLD) |
                ({} if no_previous_exist else {'app_startup_previous_exist': 'previous-name'})
        )


class ConvivaApplicationForeground(MobileOnlyEvent):
    def __init__(self):
        super().__init__()


class WebOnlyEvent(Event, ABC):
    @abstractmethod
    def __init__(self):
        event_name = self._camel_case2snake_case(self.__class__.__name__)
        super().__init__(event_name, Platform.WEB)


class ConvivaPagePing(WebOnlyEvent, FixedFrequency):
    def __init__(self):
        super().__init__()


class ConvivaPageView(WebOnlyEvent):
    def __init__(self):
        super().__init__()


# Both Mobile and Web platforms can have this kind of event
class ConvivaApplicationError(Event):
    def __init__(self, platform: Platform):
        super().__init__(self._camel_case2snake_case(self.__class__.__name__), platform)


class ConvivaVideoEvents(Event):
    _names = (['c3.sdk.custom_event', 'c3.video.custom_event'] + # can keep session alive in next 90s
              ['"c3.video.attempt"'] +
              [f"cannot-keep-session-alive-{i}" for i in range(2)])

    def __init__(self, platform: Platform):
        super().__init__(self._camel_case2snake_case(self.__class__.__name__), platform)

    def generate(self):
        basic_info = super().generate()
        return {**basic_info, 'conviva_video_events_name': random.choice(self._names)}


class ConvivaNetworkRequest(Event):
    NETWORK_REQUEST_TIME_THRESHOLD = 90000  # ms

    def __init__(self, platform: Platform):
        super().__init__(self._camel_case2snake_case(self.__class__.__name__), platform)

    def generate(self):
        basic_info = super().generate()
        return (
                basic_info |
                {
                    "response_code": random.choice(['100', '200', '300', '400', '500']),
                    "network_request_duration": str(random.randint(10, 1000) + 1)
                }
        )


def all_kinds_of_events() -> list[Type[Event]]:
    result = []

    def traverse(clazz) -> None:
        for subclass in [sc for sc in clazz.__subclasses__()]:
            if not inspect.isabstract(subclass):
                result.append(subclass)
            else:
                traverse(subclass)

    traverse(Event)
    return result


def collect_event_generators_for(platform: Platform) -> list[Event]:
    platform_event_type = MobileOnlyEvent if platform is Platform.MOB else WebOnlyEvent
    to_be_excluded_event_type = MobileOnlyEvent if platform_event_type is not MobileOnlyEvent else WebOnlyEvent
    generators = {
        eg() if issubclass(eg, platform_event_type) else eg(platform)
        for eg in all_kinds_of_events()
        if not issubclass(eg, to_be_excluded_event_type)
    }
    return list(generators | {RandomNameEvent(selected_platform) for _ in range(4)})


def generate_all_timestamps(count: int):
    time_delta = timedelta(seconds=0)
    for _ in range(count):
        time_delta += timedelta(seconds=random.randint(10, 1000) / 10.0)
        yield timestamp(time_delta)


if __name__ == '__main__':
    output_file = sys.stdout
    if len(sys.argv) < 2 or not sys.argv[1].isdigit():
        logging.error('This script only accept one integer argument, which represent the required number of entries.')
        sys.exit(1)
    else:
        required_count = int(sys.argv[1])
        default_output_path = Path(__file__).parent.parent / 'data' / 'app-analytics-metrics-demo-input.jsonl'
        output_path = sys.argv[2] if len(sys.argv) >= 3 else default_output_path
        if output_path != "-":
            output_file = open(output_path, "w")
    selected_platform = random.choice([Platform.MOB, Platform.WEB])
    logging.info(f"Generate data for the {selected_platform} platform.")
    event_generators = collect_event_generators_for(selected_platform)
    for ts in generate_all_timestamps(required_count):
        print(json.dumps({"timestamp": f"{ts}"} | random.choice(event_generators).generate()), file = output_file)