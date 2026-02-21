use jarust_interface::janus_interface::JanusInterfaceImpl;
use std::time::Duration;

pub struct JaKeepAlive {
    interface: JanusInterfaceImpl,
    session_id: u64,
    ka_interval: u32,
}

impl JaKeepAlive {
    pub fn new(interface: JanusInterfaceImpl, session_id: u64, ka_interval: u32) -> Self {
        Self {
            interface,
            session_id,
            ka_interval,
        }
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all, fields(session_id = self.session_id))]
    pub async fn start(&self) -> Result<(), jarust_interface::Error> {
        if !self.interface.has_keep_alive() {
            tracing::debug!("Keep-alive not supported");
            return Ok(());
        }
        let duration = Duration::from_secs(self.ka_interval.into());
        loop {
            jarust_rt::sleep(duration).await;
            tracing::debug!("Sending keep-alive");
            match self.interface.keep_alive(self.session_id, duration).await {
                Ok(_) => tracing::debug!("Keep-alive success"),
                Err(e) => tracing::error!("Keep-alive failed: {:?}", e),
            };
        }
    }
}
