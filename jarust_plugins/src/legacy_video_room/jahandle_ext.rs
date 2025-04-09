use super::events::PluginEvent;
use super::handle::LegacyVideoRoomHandle;
use jarust_core::prelude::*;
use std::ops::Deref;
use std::time::Duration;
use tokio::sync::mpsc;

#[async_trait::async_trait]
pub trait LegacyVideoRoom: Attach {
    type Event: TryFrom<JaResponse, Error = jarust_interface::Error> + Send + Sync + 'static;
    type Handle: From<JaHandle> + Deref<Target = JaHandle> + PluginTask;

    async fn attach_legacy_video_room(
        &self,
        timeout: Duration,
    ) -> Result<(Self::Handle, mpsc::UnboundedReceiver<Self::Event>), jarust_interface::Error> {
        let (handle, mut receiver) = self
            .attach("janus.plugin.videoroom".to_string(), timeout)
            .await?;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let task = jarust_rt::spawn("videoroom listener", async move {
            while let Some(rsp) = receiver.recv().await {
                if let Ok(event) = rsp.try_into() {
                    let _ = tx.send(event);
                };
            }
        });
        let mut handle: Self::Handle = handle.into();
        handle.assign_task(task);
        Ok((handle, rx))
    }
}

impl LegacyVideoRoom for JaSession {
    type Event = PluginEvent;
    type Handle = LegacyVideoRoomHandle;
}
