use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::RwLock as AsyncRwLock;

struct WaitFuture<K, V> {
    key: K,
    map: Arc<AsyncRwLock<IndexMap<K, V>>>,
    wakers: Arc<AsyncMutex<HashMap<K, Vec<std::task::Waker>>>>,
}

impl<K, V> Future for WaitFuture<K, V>
where
    K: Eq + Hash + Clone + Debug + Unpin,
    V: Clone + Debug + Unpin,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Check if the key exists in the map
        if this
            .map
            .try_read()
            .map(|guard| guard.contains_key(&this.key))
            .unwrap_or(false)
        {
            return Poll::Ready(());
        }

        // Add our waker to the list of wakers for this key
        if let Ok(mut wakers) = this.wakers.try_lock() {
            wakers
                .entry(this.key.clone())
                .or_insert_with(Vec::new)
                .push(cx.waker().clone());
        }

        Poll::Pending
    }
}

#[derive(Debug)]
pub struct NapMap<K, V>
where
    K: Eq + Hash + Clone + Debug + Unpin,
    V: Clone + Debug + Unpin,
{
    map: Arc<AsyncRwLock<IndexMap<K, V>>>,
    wakers: Arc<AsyncMutex<HashMap<K, Vec<std::task::Waker>>>>,
    bound: usize,
}

impl<K, V> NapMap<K, V>
where
    K: Eq + Hash + Clone + Debug + Unpin,
    V: Clone + Debug + Unpin,
{
    pub fn new(buffer: usize) -> Self {
        assert!(buffer > 0, "buffer > 0");
        tracing::trace!("Created new NapMap");
        Self {
            map: Arc::new(AsyncRwLock::new(IndexMap::with_capacity(buffer))),
            wakers: Arc::new(AsyncMutex::new(HashMap::new())),
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

        // Wake up all waiting tasks for this key
        if let Some(wakers) = self.wakers.lock().await.remove(&key) {
            for waker in wakers {
                waker.wake();
            }
            tracing::trace!("Woke up waiting tasks");
        }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip(self))]
    pub async fn get(&self, key: K) -> Option<V> {
        tracing::trace!("Getting value");
        if self.map.read().await.contains_key(&key) {
            tracing::debug!("Key present");
            return self.map.read().await.get(&key).cloned();
        }

        // Wait for the key to be available
        tracing::trace!("Waiting for key");
        WaitFuture {
            key: key.clone(),
            map: self.map.clone(),
            wakers: self.wakers.clone(),
        }
        .await;
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
