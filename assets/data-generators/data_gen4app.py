#!/usr/bin/env python3

import inspect
import json
import logging
import random
import re
import sys
from abc import ABC, abstractmethod
from datetime import timedelta
from enum import Enum
from pathlib import Path
from typing import Type

from gen_utils import random_bool, timestamp_gen

logging.basicConfig(level=logging.INFO)


class Platform(Enum):
    MOB = 1
    WEB = 2

    def __str__(self):
        return f"{self.name.lower()}"


# TODO: Not in use. Can be used to make the generated data more like real data.
class FixedFrequency(ABC):
    FIXED_TIME_INTERVAL = 40000  # ms


class Event(ABC):
    PAGE_LOAD_TIME_THRESHOLD = 90000  # ms
    SCREEN_LOAD_TIME_THRESHOLD = 60000  # ms

    EVENT_CATEGORY_POOL = [f"ec{i}" for i in range(3)]

    @abstractmethod
    def __init__(self, event_name: str, platform: Platform):
        self.event_name = event_name
        self.platform = platform
        self.basic_info = {
            "event_name": self.event_name,
            "event_category": random.choice(self.EVENT_CATEGORY_POOL),
            "platform": str(self.platform),
        }
        self.is_web = platform is Platform.WEB

    def generate(self):
        id_key = "page_id" if self.is_web else "screen_id"
        navigation_id_info = (
            {}
            if bool(random.getrandbits(3))
            else {id_key: f"nav-id-{random.randint(0, 5)}"}
        )
        load_threshold = (
            self.PAGE_LOAD_TIME_THRESHOLD
            if self.is_web
            else self.SCREEN_LOAD_TIME_THRESHOLD
        )
        load_start_end_info = (
            {}
            if random_bool()
            else self._random_time_interval("load_start", "load_end", load_threshold)
        )
        return self.basic_info | navigation_id_info | load_start_end_info

    # This time interval can be valid or invalid
    @classmethod
    def _random_time_interval(
        cls, start_key: str, end_key: str, threshold: int
    ) -> dict[str, str]:
        smaller = random.randint(0, 10)
        greater = random.randint(10, 1000)
        valid_order = random_bool()
        return {
            start_key: str(smaller if valid_order else greater),
            end_key: str(
                (greater if valid_order else smaller)
                + random.choice([0, 10, 100, threshold])
            ),
        }

    @classmethod
    def _camel_case2snake_case(cls, name: str) -> str:
        return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


class RandomNameEvent(Event):
    def __init__(self, platform: Platform):
        super().__init__("random_event_" + str(random.randint(1, 10)), platform)


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
        no_previous_exist = random_bool()
        return (
            basic_info
            | self._random_time_interval(
                "app_startup_start", "app_startup_end", self.APP_STARTUP_TIME_THRESHOLD
            )
            | (
                {}
                if no_previous_exist
                else {"app_startup_previous_exist": "previous-name"}
            )
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
    _names = (
        [
            "c3.sdk.custom_event",
            "c3.video.custom_event",
        ]  # can keep session alive in next 90s
        + ["c3.video.attempt"]
        + [f"cannot-keep-session-alive-{i}" for i in range(2)]
    )

    def __init__(self, platform: Platform):
        super().__init__(self._camel_case2snake_case(self.__class__.__name__), platform)

    def generate(self):
        basic_info = super().generate()
        return {**basic_info, "conviva_video_events_name": random.choice(self._names)}


class ConvivaNetworkRequest(Event):
    NETWORK_REQUEST_TIME_THRESHOLD = 90000  # ms

    def __init__(self, platform: Platform):
        super().__init__(self._camel_case2snake_case(self.__class__.__name__), platform)

    def generate(self):
        basic_info = super().generate()
        return basic_info | {
            "response_code": random.choice(["100", "200", "300", "400", "500"]),
            "network_request_duration": str(random.randint(10, 1000) + 1),
        }


def all_kinds_of_events() -> list[Type[Event]]:
    result = []

    def traverse(clazz) -> None:
        for subclass in list(clazz.__subclasses__()):
            if not inspect.isabstract(subclass):
                result.append(subclass)
            else:
                traverse(subclass)

    traverse(Event)
    return result


def collect_event_generators_for(platform: Platform) -> list[Event]:
    platform_event_type = MobileOnlyEvent if platform is Platform.MOB else WebOnlyEvent
    to_be_excluded_event_type = (
        MobileOnlyEvent if platform_event_type is not MobileOnlyEvent else WebOnlyEvent
    )
    result = {
        eg() if issubclass(eg, platform_event_type) else eg(platform)
        for eg in all_kinds_of_events()
        if not issubclass(eg, to_be_excluded_event_type)
    }
    # For now both platforms have 6 distinct event types, except the random
    # placeholder events.
    assert (
        len(result) == 6 if platform is Platform.WEB else 7
    ), "Please fix event data generator: some events don't show up in generated result!"
    return list(result | {RandomNameEvent(selected_platform) for _ in range(4)})


RATE_OF_KEEPING_LAST_TIMESTAMP = 0.01


def generate_all_timestamps(count: int):
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


if __name__ == "__main__":
    output_file = sys.stdout
    if len(sys.argv) < 2 or not sys.argv[1].isdigit():
        logging.error(
            "Only accept one integer argument that specifies the required number of entries."
        )
        sys.exit(1)
    else:
        required_count = int(sys.argv[1])
        sample_data_home = Path(__file__).parent.parent / "data"
        default_output_path = (
            sample_data_home / "app-analytics-metrics-demo-input.jsonl"
        )
        output_path = sys.argv[2] if len(sys.argv) >= 3 else default_output_path
        if output_path != "-":
            output_file = open(output_path, "w", encoding="utf-8")
    selected_platform = random.choice([Platform.MOB, Platform.WEB])
    logging.info("Generate data for the %s platform.", selected_platform)
    event_generators = collect_event_generators_for(selected_platform)
    for ts in generate_all_timestamps(required_count):
        event = {"timestamp": f"{ts}"} | random.choice(event_generators).generate()
        print(json.dumps(event), file=output_file)
