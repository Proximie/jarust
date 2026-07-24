use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::Notify;
use tokio::sync::RwLock as AsyncRwLock;

#[derive(Debug)]
pub struct NapMap<K, V>
where
    K: Eq + Hash + Clone + Debug,
    V: Clone + Debug,
{
    map: Arc<AsyncRwLock<IndexMap<K, V>>>,
    notifiers: Arc<AsyncMutex<HashMap<K, Arc<Notify>>>>,
    bound: usize,
}

impl<K, V> NapMap<K, V>
where
    K: Eq + Hash + Clone + Debug,
    V: Clone + Debug,
{
    pub fn new(buffer: usize) -> Self {
        assert!(buffer > 0, "buffer > 0");
        tracing::trace!("Created new NapMap");
        Self {
            map: Arc::new(AsyncRwLock::new(IndexMap::with_capacity(buffer))),
            notifiers: Arc::new(AsyncMutex::new(HashMap::new())),
            bound: buffer,
        }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip(self, value))]
    pub async fn insert(&self, key: K, value: V) {
        tracing::trace!("Inserting");

        let mut map = self.map.write().await;
        if map.len() >= self.bound {
            map.pop();
        }
        map.insert(key.clone(), value);
        drop(map);

        if let Some(notify) = self.notifiers.lock().await.remove(&key) {
            notify.notify_waiters();
            tracing::trace!("Notified waiting tasks");
        }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip(self))]
    pub async fn get(&self, key: K) -> Option<V> {
        tracing::trace!("Getting value");
        if self.map.read().await.contains_key(&key) {
            tracing::debug!("Key present");
            return self.map.read().await.get(&key).cloned();
        }

        let mut notifiers = self.notifiers.lock().await;
        let notify = notifiers
            .entry(key.clone())
            .or_insert(Arc::new(Notify::new()))
            .clone();
        drop(notifiers);

        tracing::trace!("Waiting for key");
        notify.notified().await;
        tracing::trace!("Key is available");
        self.map.read().await.get(&key).cloned()
    }

    #[allow(unused)]
    pub async fn len(&self) -> usize {
        self.map.read().await.len()
    }

    #[allow(unused)]
    pub async fn is_empty(&self) -> bool {
        self.map.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::NapMap;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn it_should_wait_until_data_is_inserted() {
        let napmap = Arc::new(NapMap::new(10));

        tokio::spawn({
            let map = napmap.clone();
            async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                map.insert("key", 7).await;
            }
        });

        let res = napmap.get("key").await.unwrap();
        assert_eq!(res, 7);
    }

    #[tokio::test]
    async fn it_should_notify_all_waiters() {
        let napmap = Arc::new(NapMap::new(10));

        tokio::spawn({
            let map = napmap.clone();
            async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                map.insert("key", 7).await;
            }
        });

        let first_handle = tokio::spawn({
            let map = napmap.clone();
            async move {
                let res = map.get("key").await.unwrap();
                assert_eq!(res, 7);
            }
        });

        let second_handle = tokio::spawn({
            let map = napmap.clone();
            async move {
                let res = map.get("key").await.unwrap();
                assert_eq!(res, 7);
            }
        });

        first_handle.await.unwrap();
        second_handle.await.unwrap();
    }

    #[tokio::test]
    async fn it_should_not_exceed_the_provided_buffer_size() {
        let napmap = Arc::new(NapMap::new(3));
        napmap.insert(1, 1).await;
        napmap.insert(2, 2).await;
        napmap.insert(3, 3).await;
        napmap.insert(4, 4).await;
        assert_eq!(napmap.len().await, 3);
    }
}
