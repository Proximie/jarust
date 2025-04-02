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
    waker_registered: bool,
}

impl<K, V> WaitFuture<K, V>
where
    K: Eq + Hash + Clone + Debug + Unpin,
    V: Clone + Debug + Unpin,
{
    fn new(
        key: K,
        map: Arc<AsyncRwLock<IndexMap<K, V>>>,
        wakers: Arc<AsyncMutex<HashMap<K, Vec<std::task::Waker>>>>,
    ) -> Self {
        Self {
            key,
            map,
            wakers,
            waker_registered: false,
        }
    }
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

        // Try to register the waker if not already registered
        if !this.waker_registered {
            if let Ok(mut wakers) = this.wakers.try_lock() {
                wakers
                    .entry(this.key.clone())
                    .or_insert_with(Vec::new)
                    .push(cx.waker().clone());
                this.waker_registered = true;
            } else {
                // If we can't get the lock, wake up immediately to try again
                cx.waker().wake_by_ref();
            }
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
        WaitFuture::new(key.clone(), self.map.clone(), self.wakers.clone()).await;
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

    #[tokio::test]
    async fn it_should_handle_concurrent_inserts_and_gets() {
        let napmap = Arc::new(NapMap::new(10));
        let mut handles = vec![];

        // Spawn multiple tasks that insert different keys
        for i in 0..5 {
            let map = napmap.clone();
            handles.push(tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(i * 100)).await;
                map.insert(format!("key{}", i), i).await;
            }));
        }

        // Spawn multiple tasks that get different keys
        for i in 0..5 {
            let map = napmap.clone();
            handles.push(tokio::spawn(async move {
                let res = map.get(format!("key{}", i)).await.unwrap();
                assert_eq!(res, i);
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn it_should_handle_rapid_successive_inserts() {
        let napmap = Arc::new(NapMap::new(5));
        let mut handles = vec![];

        for i in 0..10 {
            let map = napmap.clone();
            handles.push(tokio::spawn(async move {
                map.insert(i, i).await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(napmap.len().await, 5);

        for i in 0..5 {
            assert_eq!(napmap.get(i).await.unwrap(), i);
        }

        for i in 5..10 {
            assert_eq!(napmap.get(i).await.unwrap(), i);
        }
    }

    #[tokio::test]
    async fn it_should_handle_get_before_insert() {
        let napmap = Arc::new(NapMap::new(10));
        let mut handles = vec![];

        // Spawn a task that gets a key before it's inserted
        let map = napmap.clone();
        handles.push(tokio::spawn(async move {
            let res = map.get("key").await.unwrap();
            assert_eq!(res, 42);
        }));

        // Spawn a task that inserts the key after a delay
        let map = napmap.clone();
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            map.insert("key", 42).await;
        }));

        // Wait for both tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn it_should_handle_multiple_gets_for_same_key() {
        let napmap = Arc::new(NapMap::new(10));
        let mut handles = vec![];

        // Spawn multiple tasks that get the same key
        for _ in 0..5 {
            let map = napmap.clone();
            handles.push(tokio::spawn(async move {
                let res = map.get("key").await.unwrap();
                assert_eq!(res, 42);
            }));
        }

        // Spawn a task that inserts the key
        let map = napmap.clone();
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            map.insert("key", 42).await;
        }));

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
