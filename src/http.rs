// granular-metrics/src/http.rs

#[cfg(feature = "http")]
#[derive(serde::Serialize)]
struct Snapshot<K> {
    per_key: std::collections::HashMap<K, (u64, u64)>,
    total:  (u64, u64),
}

#[cfg(feature = "http")]
async fn handler<K>() -> impl actix_web::Responder
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static + serde::Serialize,
{
    let raw = {
        match crate::registry::auxiliary::MetricsSnapshot::<K>::fetch() {
            Ok(data) => {
                data
            },
            Err(crate::registry::auxiliary::CounterRegistryError::Error(msg)) => {
                return actix_web::HttpResponse::InternalServerError().body(msg);
            }
        }
    };

    let snap: Snapshot<K> = Snapshot {
        per_key: raw.per_key,
        total:   raw.total,
    };

    return actix_web::HttpResponse::Ok().json(snap)
}

#[cfg(feature = "http")]
pub(crate) async fn serve<K>(address: String, port: String, path: String, workers: String) -> std::io::Result<()>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static + serde::Serialize,
{
    let worker_count: usize = {
        match workers.as_str() {
            "all" => {
                num_cpus::get()
            },
            "" => {
                num_cpus::get()
            },
            other => {
                match other.parse::<usize>() {
                    Ok(parsed) => {
                        parsed
                    },
                    Err(error) => {
                        log::error!("{:?}", &error);
                        return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, error));
                    }
                }
            }
        }
    };

    let parsed_port: u16 = {
        match port.parse::<u16>() {
            Ok(parsed) => {
                parsed
            },
            Err(error) => {
                log::error!("{:?}", &error);
                return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, error));
            }
        }
    };

    let route: String = format!("/{}", path);

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .route(route.as_str(), actix_web::web::get().to(handler::<K>))
    })
    .bind((address.as_str(), parsed_port))?
    .workers(worker_count) 
    .run()
    .await
}