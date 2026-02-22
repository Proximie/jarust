use futures_util::Future;
use std::time::Duration;
use tokio::task::AbortHandle;

pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

pub fn spawn<F>(name: &str, future: F) -> JaTask
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let jh = tokio::spawn(future);
    JaTask {
        inner: jh.abort_handle(),
        task_name: name.to_owned(),
    }
}

#[derive(Debug)]
pub struct JaTask {
    inner: AbortHandle,
    pub task_name: String,
}

impl JaTask {
    pub fn cancel(&self) {
        self.inner.abort();
    }
}

impl Drop for JaTask {
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    fn drop(&mut self) {
        tracing::trace!(task_name = self.task_name, "Dropping task");
        self.cancel();
    }
}
