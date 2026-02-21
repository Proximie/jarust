use crate::prelude::*;
use jarust_rt::JaTask;
use std::time::Duration;
use tokio::sync::mpsc;

pub trait PluginTask {
    fn assign_task(&mut self, task: JaTask);
    fn cancel_task(&mut self);
}

#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
pub trait Attach {
    async fn attach(
        &self,
        plugin_id: String,
        timeout: Duration,
    ) -> Result<(JaHandle, mpsc::UnboundedReceiver<JaResponse>), jarust_interface::Error>;
}
