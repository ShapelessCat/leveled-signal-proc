use lsp_runtime::signal::SingnalProcessor;
use lsp_runtime::{UpdateContext, Timestamp};

/// A latch is a signal processor that takes a control input and a data input.
/// For each time, a latch produce the same output as the internal state.
/// When the control input becomes true, the latch change it internal state to the data input.
/// This concept borrowed from the hardware component which shares the same name. And it's widely use
/// as one bit memory in digital circuits. 
#[derive(Default)]
pub struct Latch<T: Clone>{
    data: T,
    default_value: T,
    value_forgotten_timestamp: Option<Timestamp>,
    time_to_memorize: Option<Timestamp>,
}

impl <T: Clone> Latch<T> {
    pub fn with_initial_value(data: T) -> Self {
        Self {
            default_value: data.clone(),
            data,
            value_forgotten_timestamp: None,
            time_to_memorize: None,
        }
    }
    pub fn with_forget_behavior(data: T, default: T, time_to_memorize: Timestamp) -> Self {
        Self {
            data,
            default_value: default,
            value_forgotten_timestamp: None,
            time_to_memorize: Some(time_to_memorize),
        }
    }
}

impl <T: Clone, I:Iterator> SingnalProcessor<I> for Latch<T> {
    type Input = (bool, T);
    type Output = T;
    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, &(ref set, ref value): &Self::Input) -> T {
        if *set {
            self.data = value.clone();
            if let Some(ttl) = self.time_to_memorize {
                let ts = ctx.frontier();
                self.value_forgotten_timestamp = Some(ts + ttl);
                ctx.schedule_signal_update(ttl);
            }
        } else if self.value_forgotten_timestamp.map_or(false, |what| what <= ctx.frontier()) {
            self.value_forgotten_timestamp = None;
            self.data = self.default_value.clone();
        }
        self.data.clone()
    }
}
