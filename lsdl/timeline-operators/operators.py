import functools

from lsdl.processors import SignalGenerator, SlidingWindow
from lsdl.processors.generators import Const
from lsdl.rust_code import RUST_DEFAULT_VALUE


class Operator:
    op = None
    is_measurement = False


class NullaryOperator(Operator):
    pass


class UnaryOperator(Operator):

    def __init__(self, input):
        self.input = input


class BinaryOperator(Operator):

    def __init__(self, left, right):
        self.left = left
        self.right = right


class KaryOperator(Operator):

    def __init__(self, args: list):
        self.args = args


class And(KaryOperator):
    op = "and"

    def process(self):
        return functools.reduce(lambda a, b: a & b, self.args)


class Or(KaryOperator):
    op = "or"

    def process(self):
        return functools.reduce(lambda a, b: a | b, self.args)


class Constant(NullaryOperator):
    op = "constant"

    def process(self, value):
        return Const(value)


class Count(UnaryOperator):
    op = "count"

    def process(self):
        return self.input.count_changes()


class DurationTrueT(UnaryOperator):
    op = "duration_true"
    is_measurement = True

    def process(self):
        return self.input.measure_duration_true()


class Equals(BinaryOperator):
    op = "equals"

    def process(self):
        return self.left == self.right


class Get(UnaryOperator):
    op = "get"

    def process(self, path):
        return getattr(self.input, path)


class EpochSeconds(UnaryOperator):
    op = "epoch_seconds"

    def process(self):
        return SignalGenerator(lambda_src="(timestamp, 0)").annotate_type("u64") / 1e9


class Not(UnaryOperator):
    op = "not"

    def process(self):
        return ~self.input


class FilterByValue(UnaryOperator):
    op = "filter_by_value"

    def process(self, values):
        t = None
        for v in values:
            if not t:
                t = (self.input == v)
                continue
            t = t | (self.input == v)
        return t


class Inequality(BinaryOperator):
    op = None

    def process(self, op):
        if op == "greaterThan":
            return self.left > self.right
        elif op == "greaterThanOrEqualTo":
            return self.left >= self.right
        elif op == "lessThan":
            return self.left < self.right
        elif op == "lessThanOrEqualTo":
            return self.left <= self.right


class Add(KaryOperator):
    op = "add"

    def process(self):
        return functools.reduce(lambda a, b: a + b, self.args)


class Multiply(KaryOperator):
    op = "multiply"

    def process(self):
        return functools.reduce(lambda a, b: a * b, self.args)


class Subtract(BinaryOperator):
    op = "subtract"

    def process(self):
        return self.left - self.right


class Divide(BinaryOperator):
    op = "divide"

    def process(self):
        return self.left / self.right


class Any(UnaryOperator):
    op = "any"

    def process(self, duration=-1):
        return self.input.has_been_true(duration=duration)


class PriorEvent(UnaryOperator):
    op = "prior_event"

    @staticmethod
    def prior_event(input, window_size=1, init_value=None):
        if not init_value:
            init_value = RUST_DEFAULT_VALUE
        sw = SlidingWindow(
            clock=input,
            data=input,
            window_size=window_size,
            init_value=init_value,
            emit_fn='|_, data| data.clone()'
        )
        return sw

    def process(self, window=1, initial_value=None):
        return PriorEvent.prior_event(
            self.input,
            window_size=window,
            init_value=initial_value
        )


class IfOp(KaryOperator):
    op = "if_op"

    def process(self):
        from lsdl.processors import If
        return If(self.args[0], self.args[1], self.args[2])


class DurationSinceLastEvent(UnaryOperator):
    op = "duration_since_last_event"
    is_measurement = True

    def process(self):
        return self.input.measure_duration_since_last_level()


class CumulativeFunc(UnaryOperator):
    op = None

    def process(self, op):
        from lsdl.processors.combinators import time_domain_fold
        return time_domain_fold(
            data=self.input,
            fold_method=op
        )
