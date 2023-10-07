import const
from lsdl.prelude import Const, DiffSinceCurrentLevel, If, SignalFilterBuilder, time_domain_fold
from schema import input_signal
from scope import ScopeName, session_id, navigation_id


_start = input_signal.load_start.parse('i32')
_end = input_signal.load_end.parse('i32')
_duration = _end - _start
_threshold = If(input_signal.platform == 'web',
                Const(const.PAGE_LOAD_TIME_THRESHOLD),
                Const(const.SCREEN_LOADTIME_THRESHOLD))
_is_valid_load_duration = (_start > 0) & (0 < _duration < _threshold)
_load_time = If(_is_valid_load_duration, _duration, Const(-1))

_load_time_clock = \
    SignalFilterBuilder(input_signal.event_name) \
        .filter_fn('_', 'true') \
        .then_filter(_load_time > 0) \
        .build_clock_filter()

_total_load_count = _load_time_clock.count_changes()


def fold_load_time(scope, method, init = None):
    """Summary a load time related metric in time domain.

    Use the `method` to summarize.
    """
    return time_domain_fold(
        data = _load_time,
        clock = _load_time_clock,
        init_state = init,
        fold_method = method,
        scope = scope)


# TODO: missing platform based logic!!!
def register_load_time_metrics(scope_signal, scope_name: ScopeName):
    """Build and register metrics for load time"""
    DiffSinceCurrentLevel(control = scope_signal,
                          data = _total_load_count).add_metric(f"life{scope_name.name}LoadCount")
    fold_load_time(scope_signal,
                   "max", init = 0).add_metric(f"life{scope_name.name}MaxLoadDuration")
    fold_load_time(scope_signal,
                   "sum").add_metric(f"life{scope_name.name}LoadDuration")


register_load_time_metrics(session_id, ScopeName.Session)
register_load_time_metrics(navigation_id, ScopeName.Navigation)
