// granular-metrics/src/registry/core.rs

#[derive(Clone, Copy, Debug)]
pub(crate) struct PerKeyMetrics {
    pub(crate) per_second: u64,
    pub(crate) per_minute: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TotalMetrics {
    pub(crate) total_per_second: u64,
    pub(crate) total_per_minute: u64,
}

pub(crate) struct CounterRegistry<K>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    pub(crate) raw:    dashmap::DashMap<K, std::sync::atomic::AtomicU64>,
    pub(crate) buffer: dashmap::DashMap<K, std::sync::Mutex<[u64; 60]>>,
    pub(crate) metrics: dashmap::DashMap<K, PerKeyMetrics>,
    pub(crate) totals:  std::sync::Mutex<TotalMetrics>,
}

impl<K> CounterRegistry<K>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    pub(crate) fn new() -> Self {
        Self {
            raw: dashmap::DashMap::new(),
            buffer: dashmap::DashMap::new(),
            metrics: dashmap::DashMap::new(),
            totals:  std::sync::Mutex::new(TotalMetrics {
                total_per_second: 0,
                total_per_minute: 0,
            }),
        }
    }

    pub(crate) fn increment(&self, key: K) {
        let counter: dashmap::mapref::one::RefMut<'_, K, std::sync::atomic::AtomicU64> 
        = self.raw.entry(key.clone()).or_insert_with(|| std::sync::atomic::AtomicU64::new(0));

        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        self.buffer.entry(key).or_insert_with(|| std::sync::Mutex::new([0; 60]));
    }

    pub(crate) fn spawn_per_second_congregator(self: std::sync::Arc<Self>) {
        let registry: std::sync::Arc<CounterRegistry<K>> = self.clone();
        let mut index: usize = 0usize;

        tokio::spawn(async move {
            loop {
                for entry in registry.raw.iter() {
                    let key: K = entry.key().clone();
                    let count = entry.value().swap(0, std::sync::atomic::Ordering::SeqCst);

                    if let Some(buffer_lock) = registry.buffer.get_mut(&key) {
                        buffer_lock.lock().unwrap()[index] = count;
                    }
                }

                index = (index + 1) % 60;

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    pub(crate) fn spawn_per_key_metrics_congregator(self: std::sync::Arc<Self>) {
        let registry: std::sync::Arc<CounterRegistry<K>> = self.clone();

        tokio::spawn(async move {
            loop {
                for entry in registry.buffer.iter() {
                    let key: K = entry.key().clone();
                    let buffer: std::sync::MutexGuard<'_, [u64; 60]> = entry.value().lock().unwrap();

                    let total: u64 = buffer.iter().sum();
                    let average: u64 = total / buffer.len() as u64;

                    registry.metrics.insert(key, PerKeyMetrics {
                        per_second: average,
                        per_minute: total,
                    });
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    pub(crate) fn spawn_total_metrics_congregator(self: std::sync::Arc<Self>) {
        let registry: std::sync::Arc<CounterRegistry<K>> = self.clone();

        tokio::spawn(async move {
            loop {
                let mut sum_minute: u64 = 0u64;

                for entry in registry.metrics.iter() {
                    sum_minute += entry.value().per_minute;
                }

                {
                    let mut totals: std::sync::MutexGuard<'_, TotalMetrics> = registry.totals.lock().unwrap();
                    totals.total_per_minute = sum_minute;
                    totals.total_per_second = sum_minute / 60;
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }
}