use super::params::LegacyVideoRoomCreateParams;
use super::responses::LegacyVideoRoomCreatedRsp;
use jarust_core::prelude::*;
use jarust_rt::JaTask;
use serde_json::Value;
use std::ops::Deref;
use std::time::Duration;

pub struct LegacyVideoRoomHandle {
    handle: JaHandle,
    task: Option<JaTask>,
}

impl LegacyVideoRoomHandle {
    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
    pub async fn create_room(
        &self,
        params: LegacyVideoRoomCreateParams,
        timeout: Duration,
    ) -> Result<LegacyVideoRoomCreatedRsp, jarust_interface::Error> {
        tracing::info!(plugin = "videoroom", "Sending create");
        let mut message: Value = params.try_into()?;
        message["request"] = "create".into();

        self.handle
            .send_waiton_rsp::<LegacyVideoRoomCreatedRsp>(message, timeout)
            .await
    }
}

impl PluginTask for LegacyVideoRoomHandle {
    fn assign_task(&mut self, task: JaTask) {
        self.task = Some(task);
    }

    fn cancel_task(&mut self) {
        if let Some(task) = self.task.take() {
            task.cancel()
        };
    }
}

impl From<JaHandle> for LegacyVideoRoomHandle {
    fn from(handle: JaHandle) -> Self {
        Self { handle, task: None }
    }
}

impl Deref for LegacyVideoRoomHandle {
    type Target = JaHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for LegacyVideoRoomHandle {
    fn drop(&mut self) {
        self.cancel_task();
    }
}
