// granular-metrics/src/lib.rs

pub(crate) mod registry;
pub(crate) mod http;

#[cfg(not(feature = "http"))]
pub fn init<K>()
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    let registry: std::sync::Arc<crate::registry::core::CounterRegistry<K>> = std::sync::Arc::new(crate::registry::core::CounterRegistry::<K>::new());

    registry.clone().spawn_per_second_congregator();

    registry.clone().spawn_per_key_metrics_congregator();

    registry.clone().spawn_total_metrics_congregator();

    crate::registry::auxiliary::REGISTRIES.insert(std::any::TypeId::of::<K>(), Box::new(registry));
}

#[cfg(feature = "http")]
pub fn init<K>(address: &str, port: &str, path: &str, workers: &str)
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static + serde::Serialize,
{
    let registry: std::sync::Arc<crate::registry::core::CounterRegistry<K>> = std::sync::Arc::new(crate::registry::core::CounterRegistry::<K>::new());

    registry.clone().spawn_per_second_congregator();

    registry.clone().spawn_per_key_metrics_congregator();

    registry.clone().spawn_total_metrics_congregator();

    crate::registry::auxiliary::REGISTRIES.insert(std::any::TypeId::of::<K>(), Box::new(registry));

    let listen_address: String = address.to_string();
    let num_of_workers: String = workers.to_string();
    let listen_port: String = port.to_string();
    let url_path: String = path.to_string();

    {
        match std::thread::Builder::new()
            .name("metrics-http".into())
            .spawn(|| {
                let system = actix_web::rt::System::new();
                system.block_on(async {
                    match crate::http::serve::<K>(listen_address, listen_port, url_path, num_of_workers).await {
                        Ok(_ok) => {},
                        Err(error) => {
                            log::error!("{:?}", error);
                            std::process::exit(1);
                        }
                    }
                });
            }) 
        {
            Ok(_ok) => {},
            Err(error) => {
                log::error!("{:?}", error);
                std::process::exit(1);
            }
        }
    }
}

pub fn increment<K>(key: K)
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{   
    let registry: Result<std::sync::Arc<registry::core::CounterRegistry<K>>, registry::auxiliary::CounterRegistryError> = crate::registry::auxiliary::get_registry::<K>();
    match registry {
        Ok(registry) => {
            registry.increment(key);
        },
        Err(error) => {
            log::error!("increment failed: Was not able to add to count. Error: {:?}", error);
        } 
    }
}

pub fn fetch<K>() -> crate::registry::auxiliary::MetricsSnapshot<K>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    let data: registry::auxiliary::MetricsSnapshot<K> = {
        match crate::registry::auxiliary::MetricsSnapshot::<K>::fetch() {
            Ok(snap) => {
                snap
            },
            Err(error) => {
                log::error!("fetch failed: Will return an empty value. Error: {:?}", error);
                
                let empty: registry::auxiliary::MetricsSnapshot<K> = crate::registry::auxiliary::MetricsSnapshot {
                    per_key: std::collections::HashMap::new(),
                    total:   (0, 0),
                };

                return empty;
            }
        }
    };

    return data;
}