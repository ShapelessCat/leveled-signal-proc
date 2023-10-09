import const
from lsdl.prelude import Const, DiffSinceCurrentLevel, If, SignalFilterBuilder, time_domain_fold
from schema import input_signal
from scope import ScopeName, session_id, navigation_id


_start = input_signal.load_start.parse('i32')
_end = input_signal.load_end.parse('i32')
_is_mob = input_signal.platform == 'mob'
_threshold = If(_is_mob,
                Const(const.SCREEN_LOADTIME_THRESHOLD),
                Const(const.PAGE_LOAD_TIME_THRESHOLD))

_is_valid_load_duration = (_start > 0) & (0 < _end - _start < _threshold)

_valid_load_duration_clock =\
    SignalFilterBuilder(_is_valid_load_duration,
                        input_signal.load_start.clock())\
        .filter_true().build_clock_filter()

_previous_start = _start.prior_value(_valid_load_duration_clock)
_previous_end = _end.prior_value(_valid_load_duration_clock)

_load_time = If(
    _is_mob,
    _end - _start,
    If((_start > _previous_start) & (_end > _previous_end), _start - _end, Const(-1))
)

_load_time_clock = SignalFilterBuilder(_load_time > 0, _valid_load_duration_clock)\
    .filter_true()\
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



# _start_end = If(_is_valid_load_duration,
#                 make_tuple(_start, _end),
#                 Const((-1, -1), Tuple(Integer(), Integer())))

# _load_time =\
#     make_tuple(_is_mob, _start_end.prior_value(), _start_end)\
#         .map(
#             '(is_mob: bool, (ps: i32, pe: i32), (s: i32, e: i32))',
#             '''
#             if is_mob {
#                 if s > 0 {e - s} else {-1}
#             } else {
#                 if s > ps && e > pe {e - s} else {-1}
#             }
#             '''
#         )
