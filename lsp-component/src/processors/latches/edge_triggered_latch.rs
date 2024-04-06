use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use lsp_runtime::signal_api::SignalProcessor;
use lsp_runtime::Duration;
use lsp_runtime::{context::UpdateContext, signal_api::Patchable};

use super::retention::{KeepForever, Retention, TimeToLive};

#[derive(Default, Debug, Serialize)]
pub struct EdgeTriggeredLatch<Control, Data, RetentionPolicy: Retention<Data> = KeepForever> {
    last_control_level: Control,
    data: Data,
    retention: RetentionPolicy,
}

impl<C: Default, D> EdgeTriggeredLatch<C, D> {
    pub fn with_initial_value(data: D) -> Self {
        Self {
            last_control_level: Default::default(),
            data,
            retention: KeepForever,
        }
    }
}

impl<C, D> EdgeTriggeredLatch<C, D, TimeToLive<D>>
where
    C: Default,
    D: Clone + Serialize + DeserializeOwned,
{
    pub fn with_forget_behavior(data: D, default: D, time_to_memorize: Duration) -> Self {
        Self {
            data,
            last_control_level: Default::default(),
            retention: TimeToLive {
                default_value: default,
                value_forgotten_timestamp: 0,
                time_to_live: time_to_memorize,
            },
        }
    }
}

impl<'a, I, C, D, R> SignalProcessor<'a, I> for EdgeTriggeredLatch<C, D, R>
where
    I: Iterator,
    C: Clone + PartialEq,
    D: Clone,
    R: Retention<D>,
{
    type Input = (C, D);
    type Output = D;
    #[inline(always)]
    fn update(&mut self, ctx: &mut UpdateContext<I>, (control, value): &'a Self::Input) -> D {
        let should_set = if &self.last_control_level != control {
            self.last_control_level = control.clone();
            true
        } else {
            false
        };
        if should_set {
            self.data = value.clone();
            if let Some(ttl) = self.retention.drop_timestamp(ctx.frontier()) {
                ctx.schedule_signal_update(ttl);
            }
        } else if let Some(value) = self.retention.should_drop(ctx.frontier()) {
            self.data = value;
        }
        self.data.clone()
    }
}

#[derive(Deserialize)]
pub struct EdgeTriggeredLatchState<Control, Data, RetentionPolicy> {
    last_control_level: Control,
    data: Data,
    retention: RetentionPolicy,
}

impl<C, D, R: Retention<D>> Patchable for EdgeTriggeredLatch<C, D, R>
where
    C: Serialize + DeserializeOwned,
    D: Serialize + DeserializeOwned,
    R: Serialize + DeserializeOwned,
{
    type State = EdgeTriggeredLatchState<C, D, R>;

    fn patch_from(&mut self, state: Self::State) {
        self.last_control_level = state.last_control_level;
        self.data = state.data;
        self.retention = state.retention;
    }
}
