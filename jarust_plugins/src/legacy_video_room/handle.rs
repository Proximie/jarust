use super::params::LegacyVideoRoomCreateParams;
use super::params::LegacyVideoRoomExistsParams;
use super::params::LegacyVideoRoomKickParams;
use super::params::LegacyVideoRoomPublisherConfigureParams;
use super::params::LegacyVideoRoomPublisherJoinAndConfigureParams;
use super::params::LegacyVideoRoomPublisherJoinParams;
use super::responses::LegacyVideoRoomCreatedRsp;
use crate::legacy_video_room::responses::LegacyVideoRoomExistsRsp;
use jarust_core::prelude::*;
use jarust_interface::japrotocol::Jsep;
use jarust_rt::JaTask;
use serde_json::Value;
use std::ops::Deref;
use std::time::Duration;

pub struct LegacyVideoRoomHandle {
    handle: JaHandle,
    task: Option<JaTask>,
}

// sync
impl LegacyVideoRoomHandle {
    /// Create a new video room dynamically with the given configuration,
    /// as an alternative to using the configuration file
    ///
    /// ### Note:
    /// Random room number will be used if `room` is `None`
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

    /// Check whether a room exists
    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
    pub async fn exists(
        &self,
        params: LegacyVideoRoomExistsParams,
        timeout: Duration,
    ) -> Result<bool, jarust_interface::Error> {
        tracing::info!(plugin = "videoroom", "Sending exists");
        let mut message: Value = params.try_into()?;
        message["request"] = "exists".into();
        let response = self
            .handle
            .send_waiton_rsp::<LegacyVideoRoomExistsRsp>(message, timeout)
            .await?;
        Ok(response.exists)
    }

    /// Kicks a participants out of a room
    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
    pub async fn kick(
        &self,
        params: LegacyVideoRoomKickParams,
        timeout: Duration,
    ) -> Result<(), jarust_interface::Error> {
        tracing::info!(plugin = "videoroom", "Sending kick");
        let mut message: Value = params.try_into()?;
        message["request"] = "kick".into();

        self.handle.send_waiton_rsp::<()>(message, timeout).await
    }
}

// async
impl LegacyVideoRoomHandle {
    pub async fn publisher_join(
        &self,
        params: LegacyVideoRoomPublisherJoinParams,
        jsep: Option<Jsep>,
        timeout: Duration,
    ) -> Result<String, jarust_interface::Error> {
        let mut message: Value = params.try_into()?;
        message["request"] = "join".into();
        message["ptype"] = "publisher".into();

        match jsep {
            None => self.handle.send_waiton_ack(message, timeout).await,
            Some(jsep) => {
                self.handle
                    .send_waiton_ack_with_jsep(message, jsep, timeout)
                    .await
            }
        }
    }

    pub async fn configure_publisher(
        &self,
        params: LegacyVideoRoomPublisherConfigureParams,
        timeout: Duration,
    ) -> Result<(), jarust_interface::Error> {
        let mut message: Value = params.try_into()?;
        message["request"] = "configure".into();
        self.handle.send_waiton_ack(message, timeout).await?;
        Ok(())
    }

    pub async fn publisher_join_and_configure(
        &self,
        params: LegacyVideoRoomPublisherJoinAndConfigureParams,
        jsep: Option<Jsep>,
        timeout: Duration,
    ) -> Result<String, jarust_interface::Error> {
        let mut message: Value = params.try_into()?;
        message["request"] = "joinandconfigure".into();
        message["ptype"] = "publisher".into();
        match jsep {
            None => self.handle.send_waiton_ack(message, timeout).await,
            Some(jsep) => {
                self.handle
                    .send_waiton_ack_with_jsep(message, jsep, timeout)
                    .await
            }
        }
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
