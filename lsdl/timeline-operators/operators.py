import functools
from lsdl.const import Const


class Operator(object):
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
        return (self.left == self.right)
    

class Get(UnaryOperator):
    op = "get"

    def process(self, path):
        return getattr(self.input, path)


# class EpochSeconds(UnaryOperator):
#     op = "epoch_seconds"
     
#     def process(self):
#         return getattr(self.input, "_timestamp_key")


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
            return (self.left > self.right)
        elif op == "greaterThanOrEqualTo":
            return (self.left >= self.right)
        elif op == "lessThan":
            return (self.left < self.right)
        elif op == "lessThanOrEqualTo":
            return (self.left <= self.right)


class Add(KaryOperator):
    op = "add"

    def process(self):
        return functools.reduce(lambda a, b: a + b, self.args)


class Multiply(KaryOperator):
    op = "multiply"

    def process(self):
        return functools.reduce(lambda a, b: a * b, self.args)


class Substract(BinaryOperator):
    op = "substract"

    def process(self):
        return (self.left - self.right)
    

class Divide(BinaryOperator):
    op = "divide"

    def process(self):
        return (self.left / self.right)


class Any(UnaryOperator):
    op = "any"

    def process(self):
        return self.input.has_been_true()
    

class PriorEvent(UnaryOperator):
    op = "prior_event"

    def process(self, window=1, initial_value=None):
        return self.input.prior_event(window_size=window, init_value=initial_value)


class IfOp(KaryOperator):
    op = "if_op"

    def process(self):
        from lsdl.signal_processors import If
        return If(self.args[0], self.args[1], self.args[2])


class DurationSinceLastEvent(UnaryOperator):
    op = "duration_since_last_event"
    is_measurement = True

    def process(self):
        return self.input.measure_duration_since_last_level()


class CummulativeFunc(UnaryOperator):
    op = None

    def process(self, op):
        from lsdl.prelude import time_domain_fold
        return time_domain_fold(
            data=self.input,
            fold_method=op
        )
