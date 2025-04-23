// granular-metrics/src/registry/auxiliary.rs

pub(crate) static REGISTRIES: once_cell::sync::Lazy<dashmap::DashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>> 
= once_cell::sync::Lazy::new(|| dashmap::DashMap::new());

#[derive(Debug)]
pub struct MetricsSnapshot<K> {
    pub per_key: std::collections::HashMap<K, (u64, u64)>,
    pub total: (u64, u64),
}

#[derive(Debug)]
pub(crate) enum CounterRegistryError {
    Error(String),
}

impl<K> MetricsSnapshot<K>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    pub(crate) fn fetch() -> Result<Self, CounterRegistryError> {

        let registry: std::sync::Arc<super::core::CounterRegistry<K>> = {
            match get_registry::<K>() {
                Ok(reg) => {
                    reg
                },
                Err(CounterRegistryError::Error(msg)) => {
                    return Err(CounterRegistryError::Error(msg));
                }
            }
        };

        let mut per_key: std::collections::HashMap<K, (u64, u64)> = std::collections::HashMap::new();

        for entry in registry.metrics.iter() {
            let p_k_m: crate::registry::core::PerKeyMetrics = *entry.value();             
            per_key.insert(entry.key().clone(), (p_k_m.per_second, p_k_m.per_minute));
        }

        let total: crate::registry::core::TotalMetrics = *registry.totals.lock().unwrap();

        return Ok(MetricsSnapshot { per_key, total: (total.total_per_second, total.total_per_minute) })
    }
}

pub(crate) fn get_registry<K>() -> Result<std::sync::Arc<crate::registry::core::CounterRegistry<K>>, CounterRegistryError> 
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    let registry_any_ref: dashmap::mapref::one::Ref<'_, std::any::TypeId, Box<dyn std::any::Any + Send + Sync>> = {
        match REGISTRIES.get(&std::any::TypeId::of::<K>()) {
            Some(any) => {
                any
            },
            None => {
                log::error!("CounterRegistry<{}> was not initialized. Please call init::<{}>() before incrementing.", std::any::type_name::<K>(), std::any::type_name::<K>());
                return Err(CounterRegistryError::Error(format!("CounterRegistry<{}> was not initialized. Please call init::<{}>() before incrementing.", std::any::type_name::<K>(), std::any::type_name::<K>())));
            }
        }
    };

    let registry: &std::sync::Arc<crate::registry::core::CounterRegistry<K>> = {
        match registry_any_ref.downcast_ref::<std::sync::Arc<crate::registry::core::CounterRegistry<K>>>() {
            Some(reg) => {
                reg
            },
            None => {
                log::error!("Stored registry for <{}> could not be downcast to CounterRegistry<{}>.", std::any::type_name::<K>(), std::any::type_name::<K>());
                return Err(CounterRegistryError::Error(format!("Stored registry for <{}> could not be downcast to CounterRegistry<{}>.", std::any::type_name::<K>(), std::any::type_name::<K>())));
            }
        }
    };

    return Ok(registry.clone());
}