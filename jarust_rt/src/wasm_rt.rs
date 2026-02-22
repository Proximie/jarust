use futures_util::Future;
use futures_util::FutureExt;
use std::time::Duration;

pub async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

pub fn spawn<F>(name: &str, future: F) -> JaTask
where
    F: Future + 'static,
{
    wasm_bindgen_futures::spawn_local(future.map(|_| ()));
    JaTask {
        task_name: name.to_owned(),
    }
}

#[derive(Debug)]
pub struct JaTask {
    pub task_name: String,
}

impl JaTask {
    pub fn cancel(&self) {
        // wasm_bindgen_futures::spawn_local does not support cancellation
    }
}

impl Drop for JaTask {
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    fn drop(&mut self) {
        tracing::trace!(task_name = self.task_name, "Dropping task");
        self.cancel();
    }
}
