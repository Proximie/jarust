use super::msg_options::StartOptions;
use jarust::prelude::*;
use jarust_rt::JaTask;
use jarust_transport::japrotocol::EstablishmentProtocol;
use std::ops::Deref;
use std::time::Duration;

pub struct EchoTestHandle {
    handle: JaHandle,
    task: Option<JaTask>,
}

impl EchoTestHandle {
    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all, fields(session_id = self.handle.session_id(), handle_id = self.handle.id()))]
    pub async fn start(&self, options: StartOptions) -> JaResult<()> {
        tracing::info!(plugin = "echotest", "Sending start");
        self.handle.fire_and_forget(options.try_into()?).await
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all, fields(session_id = self.handle.session_id(), handle_id = self.handle.id()))]
    pub async fn start_with_est(
        &self,
        options: StartOptions,
        establishment: EstablishmentProtocol,
        timeout: Duration,
    ) -> JaResult<()> {
        tracing::info!(plugin = "echotest", "Sending start with establishment");
        self.send_waiton_ack_with_est(options.try_into()?, establishment, timeout)
            .await?;
        Ok(())
    }
}

impl PluginTask for EchoTestHandle {
    fn assign_task(&mut self, task: JaTask) {
        self.task = Some(task);
    }

    fn cancel_task(&mut self) {
        if let Some(task) = self.task.take() {
            task.cancel();
        };
    }
}

impl From<JaHandle> for EchoTestHandle {
    fn from(handle: JaHandle) -> Self {
        Self { handle, task: None }
    }
}

impl Deref for EchoTestHandle {
    type Target = JaHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for EchoTestHandle {
    fn drop(&mut self) {
        self.cancel_task();
    }
}

impl Clone for EchoTestHandle {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            task: None,
        }
    }
}
