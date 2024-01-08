use lsp_runtime::{signal::SignalProcessor, Duration, Timestamp, UpdateContext};

pub trait SignalFunc<T> {
    fn call(&mut self, ts: Timestamp) -> (T, Timestamp);
}

impl<T, F: FnMut(Timestamp) -> (T, Timestamp)> SignalFunc<T> for F {
    fn call(&mut self, ts: Timestamp) -> (T, Timestamp) {
        self(ts)
    }
}

pub struct ConstSignalFunc<T>(pub T);

impl<T: Clone> SignalFunc<T> for ConstSignalFunc<T> {
    fn call(&mut self, _: Timestamp) -> (T, Timestamp) {
        (self.0.clone(), Timestamp::MAX)
    }
}

/// A signal generator is a leveled signal processor that produces leveled signal based on
/// timestamps.
/// The `SignalFunc` is a lambda that is called to determine the current level of the signal it
/// receives a timestamp for now and returns a tuple of signal level and the timestamp when current
/// level ends.
pub struct SignalGenerator<SignalFunc = ConstSignalFunc<i32>, SignalType = i32> {
    signal_func: SignalFunc,
    last_value: SignalType,
    until_ts: Timestamp,
}

impl<F, O> SignalGenerator<F, O>
where
    F: FnMut(Timestamp) -> (O, Timestamp),
    O: Default,
{
    pub fn new(signal_func: F) -> Self {
        Self {
            signal_func,
            last_value: O::default(),
            until_ts: 0,
        }
    }
}

impl SignalGenerator {
    pub fn square_wave(
        period: Duration,
        phase: Timestamp,
    ) -> SignalGenerator<impl FnMut(u64) -> (bool, u64), bool> {
        SignalGenerator::new(move |now| {
            (
                ((now - phase) / period) & 1 > 0,
                period - (now + period - phase) % period,
            )
        })
    }

    pub fn raising_level(
        mut start: i64,
        step: i64,
        duration: Duration,
        phase: Timestamp,
    ) -> SignalGenerator<impl FnMut(u64) -> (i64, u64), i64> {
        let mut next_level_starts = None;
        SignalGenerator::new(move |now| {
            if let Some(right) = next_level_starts {
                if right <= now {
                    start += step;
                }
            } else {
                next_level_starts = Some(now + phase + duration - now % duration);
            }
            (start, duration - (now + duration - phase) % duration)
        })
    }
}

impl<'a, F, O, I> SignalProcessor<'a, I> for SignalGenerator<F, O>
where
    F: FnMut(Timestamp) -> (O, Timestamp),
    I: Iterator,
    O: Clone,
{
    type Input = ();

    type Output = O;

    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, (): ()) -> Self::Output {
        if self.until_ts <= ctx.frontier() {
            let (last_value, time_diff) = (self.signal_func)(ctx.frontier());
            self.until_ts = time_diff + ctx.frontier();
            self.last_value = last_value;
            if ctx.frontier() < self.until_ts {
                ctx.schedule_signal_update(time_diff);
            }
        }
        self.last_value.clone()
    }
}

#[cfg(test)]
mod test {
    use lsp_runtime::signal::SignalProcessor;

    use crate::test::{create_lsp_context_for_test, TestSignalBag};

    use super::SignalGenerator;

    #[test]
    fn test_square_wave() {
        let mut ctx = create_lsp_context_for_test();
        let mut square_wave = SignalGenerator::square_wave(10, 0);
        let mut state = TestSignalBag::default();

        while let Some(moment) = ctx.next_event(&mut state) {
            let mut uc = ctx.borrow_update_context();
            let value = square_wave.update(&mut uc, ());
            assert_eq!(value, (moment.timestamp() / 10) % 2 == 1);
        }
    }

    #[test]
    fn test_raising_level() {
        let mut ctx = create_lsp_context_for_test();
        let mut raising_level = SignalGenerator::raising_level(0, 1, 10, 0);
        let mut state = TestSignalBag::default();

        while let Some(moment) = ctx.next_event(&mut state) {
            let mut uc = ctx.borrow_update_context();
            let value = raising_level.update(&mut uc, ());
            assert_eq!(value, moment.timestamp() as i64 / 10);
        }
    }

    #[test]
    fn test_fib_seq() {
        let mut ctx = create_lsp_context_for_test();
        let mut a = 0;
        let mut b = 1;
        let mut fib_seq = SignalGenerator::new(move |_now| {
            let c = a + b;
            a = b;
            b = c;
            (a, 100)
        });
        let mut state = TestSignalBag::default();

        let mut fa = 0;
        let mut fb = 1;

        while let Some(moment) = ctx.next_event(&mut state) {
            let mut uc = ctx.borrow_update_context();
            let value = fib_seq.update(&mut uc, ());

            if moment.timestamp() % 100 == 0 {
                let c = fa + fb;
                fa = fb;
                fb = c;
            }

            assert_eq!(value, fa);
        }
    }
}
