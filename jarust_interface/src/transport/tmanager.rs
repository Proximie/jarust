use super::ringbuf_map::RingBufMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub(crate) struct TransactionManager {
    inner: Arc<RwLock<RingBufMap<String, String>>>,
}

impl TransactionManager {
    #[tracing::instrument(level = tracing::Level::TRACE)]
    pub(crate) fn new(capacity: usize) -> Self {
        tracing::trace!("Creating new transaction manager");
        let transactions = RingBufMap::new(capacity);
        let inner = Arc::new(RwLock::new(transactions));
        Self { inner }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip(self))]
    pub(crate) async fn get(&self, id: &str) -> Option<String> {
        tracing::trace!("Getting transaction");
        self.inner.read().await.get(&id.into()).cloned()
    }

    #[tracing::instrument(parent = None, skip(self))]
    pub(crate) async fn insert(&self, id: &str, transaction: &str) {
        tracing::trace!("Inserting transaction");
        self.inner.write().await.put(id.into(), transaction.into());
    }
}
