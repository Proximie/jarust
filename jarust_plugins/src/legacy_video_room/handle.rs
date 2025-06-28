use super::params::LegacyVideoRoomCreateParams;
use super::params::LegacyVideoRoomExistsParams;
use super::params::LegacyVideoRoomKickParams;
use super::params::LegacyVideoRoomPublisherConfigureParams;
use super::params::LegacyVideoRoomPublisherJoinAndConfigureParams;
use super::params::LegacyVideoRoomPublisherJoinParams;
use super::params::LegacyVideoRoomSubscriberConfigureParams;
use super::params::LegacyVideoRoomSubscriberJoinParams;
use super::responses::LegacyVideoRoomCreatedRsp;
use crate::legacy_video_room::responses::LegacyVideoRoomExistsRsp;
use jarust_core::prelude::*;
use jarust_interface::japrotocol::Jsep;
use jarust_rt::JaTask;
use serde_json::json;
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

        self.handle
            .send_waiton_rsp::<Value>(message, timeout)
            .await?;
        Ok(())
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

    pub async fn publisher_configure(
        &self,
        params: LegacyVideoRoomPublisherConfigureParams,
        timeout: Duration,
    ) -> Result<String, jarust_interface::Error> {
        let mut message: Value = params.try_into()?;
        message["request"] = "configure".into();
        self.handle.send_waiton_ack(message, timeout).await
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

    pub async fn subscriber_join(
        &self,
        params: LegacyVideoRoomSubscriberJoinParams,
        timeout: Duration,
    ) -> Result<String, jarust_interface::Error> {
        let mut message: Value = params.try_into()?;
        message["request"] = "join".into();
        message["ptype"] = "subscriber".into();
        self.handle.send_waiton_ack(message, timeout).await
    }

    pub async fn subscriber_configure(
        &self,
        params: LegacyVideoRoomSubscriberConfigureParams,
        timeout: Duration,
    ) -> Result<String, jarust_interface::Error> {
        let mut message: Value = params.try_into()?;
        message["request"] = "configure".into();
        self.handle.send_waiton_ack(message, timeout).await
    }

    /// Complete the setup of the PeerConnection for a subscriber
    ///
    /// The subscriber is supposed to send a JSEP SDP answer back to the plugin by the means of this request,
    /// which in this case MUST be associated with a JSEP SDP answer but otherwise requires no arguments.
    pub async fn start(
        &self,
        jsep: Jsep,
        timeout: Duration,
    ) -> Result<String, jarust_interface::Error> {
        self.handle
            .send_waiton_ack_with_jsep(json!({"request": "start"}), jsep, timeout)
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
