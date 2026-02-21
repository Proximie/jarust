use crate::jahandle::JaHandle;
use crate::jahandle::NewHandleParams;
use crate::jakeepalive::JaKeepAlive;
use crate::prelude::*;
use jarust_interface::janus_interface::JanusInterfaceImpl;
use jarust_rt::JaTask;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Shared {
    id: u64,
    interface: JanusInterfaceImpl,
}

#[derive(Debug, Default)]
pub struct Exclusive {
    task: Option<JaTask>,
}

#[derive(Debug)]
struct InnerSession {
    shared: Shared,
    exclusive: Mutex<Exclusive>,
}

#[derive(Clone, Debug)]
pub struct JaSession {
    inner: Arc<InnerSession>,
}

pub struct NewSessionParams {
    pub session_id: u64,
    pub ka_interval: u32,
    pub interface: JanusInterfaceImpl,
}

impl JaSession {
    pub(crate) async fn new(params: NewSessionParams) -> Self {
        let shared = Shared {
            id: params.session_id,
            interface: params.interface.clone(),
        };
        let exclusive = Mutex::new(Exclusive::default());
        let session = Self {
            inner: Arc::new(InnerSession { shared, exclusive }),
        };

        let jakeepalive = JaKeepAlive::new(params.interface, params.session_id, params.ka_interval);

        let keepalive_task =
            jarust_rt::spawn("KeepAlive task", async move { jakeepalive.start().await });

        session.inner.exclusive.lock().await.task = Some(keepalive_task);

        session
    }
}

impl JaSession {
    /// Destroy the current session
    ///
    /// Similar to [`destroy`](Self::destroy) but it borrows the session instead of consuming it
    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all, fields(session_id = self.inner.shared.id))]
    pub async fn destroy(&self, timeout: Duration) -> Result<(), jarust_interface::Error> {
        tracing::info!("Destroying session");
        let session_id = self.inner.shared.id;
        self.inner
            .shared
            .interface
            .destroy(session_id, timeout)
            .await?;
        Ok(())
    }

    /// Destroy the current session
    ///
    /// Similar to [`destroy`](Self::destroy) but consumes the session
    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all, fields(session_id = self.inner.shared.id))]
    pub async fn into_destroy(self, timeout: Duration) -> Result<(), jarust_interface::Error> {
        tracing::info!("Destroying and dropping session");
        let session_id = self.inner.shared.id;
        self.inner
            .shared
            .interface
            .destroy(session_id, timeout)
            .await?;
        Ok(())
    }
}

#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
impl Attach for JaSession {
    /// Attach a plugin to the current session
    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all, fields(session_id = self.inner.shared.id))]
    async fn attach(
        &self,
        plugin_id: String,
        timeout: Duration,
    ) -> Result<(JaHandle, mpsc::UnboundedReceiver<JaResponse>), jarust_interface::Error> {
        tracing::info!(plugin = &plugin_id, "Attaching new handle");
        let session_id = self.inner.shared.id;
        let (handle_id, event_receiver) = self
            .inner
            .shared
            .interface
            .attach(session_id, plugin_id, timeout)
            .await?;

        let handle = JaHandle::new(NewHandleParams {
            handle_id,
            session_id,
            interface: self.inner.shared.interface.clone(),
        })
        .await;
        tracing::info!(id = handle_id, "Handle created");
        Ok((handle, event_receiver))
    }
}

impl Drop for Exclusive {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.cancel()
        }
    }
}
