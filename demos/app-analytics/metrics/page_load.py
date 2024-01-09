import const
from lsdl.prelude import Cond, Const, If, SignalFilterBuilder, time_domain_fold
from schema import input_signal
from scope import ScopeName, session_id, navigation_id


_start = input_signal.load_start.parse('i32')
_end = input_signal.load_end.parse('i32')
_is_mob = input_signal.platform == const.PLATFORM_MOBILE
_threshold = If(_is_mob,
                Const(const.SCREEN_LOADTIME_THRESHOLD),
                Const(const.PAGE_LOAD_TIME_THRESHOLD))

_is_valid_load_duration = (_start > 0) & (0 < _end - _start < _threshold)

_valid_load_duration_clock = (
    SignalFilterBuilder(_is_valid_load_duration,
                        input_signal.load_start.clock())
    .filter_true()
    .build_clock_filter()
)

_previous_start = _start.prior_value(_valid_load_duration_clock)
_previous_end = _end.prior_value(_valid_load_duration_clock)

_load_time = Cond(
    (_is_mob, _end - _start),
    [((_start > _previous_start) & (_end > _previous_end), _end - _start)],
    Const(-1)
)

_load_time_clock = SignalFilterBuilder(
    _load_time > 0,
    _valid_load_duration_clock
).filter_true().build_clock_filter()

_total_load_count = _load_time_clock.count_changes()


def fold_load_time(scope, method, init=None):
    """Summary a load time related metric in time domain.

    Use the `method` to summarize.
    """
    return time_domain_fold(
        data=_load_time,
        clock=_load_time_clock,
        init_state=init,
        fold_method=method,
        scope=scope)


def register_load_time_metrics(scope_signal, scope_name: ScopeName):
    """Build and register metrics for load time"""
    scope = scope_name.name.lower()
    _total_load_count\
        .peek()\
        .scope(scope_signal)\
        .add_metric(f"life_{scope}_load_count")
    fold_load_time(
        scope_signal,
        "max",
        init=0
    ).add_metric(f"life_{scope}_max_load_duration")
    fold_load_time(
        scope_signal,
        "sum"
    ).add_metric(f"life_{scope}_load_duration")


register_load_time_metrics(session_id, ScopeName.Session)
register_load_time_metrics(navigation_id, ScopeName.Navigation)
