use std::any::Any;

use lsp_runtime::{Timestamp, signal::SingnalProcessor, UpdateContext};

/// A signal generator is a leveled signal processor that produce leveled signal
/// based on the timestamp. 
/// The SignalFunc is a lambda that is called to determine the current level of the signal
/// it recieve a timestamp for now and returns a tuple of signal level and the timestamp when current level ends.
pub struct SignalGenerator<SignalFunc, SignalType> {
    signal_func: SignalFunc,
    last_value: SignalType,
    until_ts: Timestamp,
}

impl <F, O> SignalGenerator<F, O> 
where
    F: FnMut(Timestamp) -> (O, Timestamp),
    O: Default
{
    pub fn new(signal_func: F) -> Self {
        Self {
            signal_func,
            last_value: O::default(),
            until_ts: 0,
        }
    }
    pub fn square_wave(period: Timestamp, phase: Timestamp) -> impl Any {
        SignalGenerator::new(move |now| {
            (((now - phase) / period) & 1 > 0, period - (now + period - phase) % period)
        })
    }
    pub fn raising_level(mut start: i64, step: i64, duration: Timestamp, phase: Timestamp) -> impl Any {
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

impl <'a, F, O, I> SingnalProcessor<'a, I> for SignalGenerator<F, O>
where
    F: FnMut(Timestamp) -> (O, Timestamp),
    I: Iterator,
    O: Clone
{
    type Input = ();

    type Output = O;

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
