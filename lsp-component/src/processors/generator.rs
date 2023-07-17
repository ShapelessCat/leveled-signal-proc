use lsp_runtime::{Timestamp, signal::SingnalProcessor, UpdateContext};

/// A signal generator is a leveled signal processor that produce leveled signal
/// based on the timestamp. 
pub struct SignalGenerator<F, O> {
    signal_func: F,
    last_value: O,
    until_ts: Timestamp,
}

impl <F, O> SignalGenerator<F, O> 
where
    F: Fn(Timestamp) -> (O, Timestamp),
    O: Default
{
    pub fn new(signal_func: F) -> Self 
    {
        Self {
            signal_func,
            last_value: O::default(),
            until_ts: 0,
        }
    }
}

impl <F, O, I> SingnalProcessor<I> for SignalGenerator<F, O>
where
    F: Fn(Timestamp) -> (O, Timestamp),
    I: Iterator,
    O: Clone
{
    type Input = ();

    type Output = O;

    fn update(&mut self, ctx: &mut UpdateContext<I>, (): &Self::Input) -> Self::Output {
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
