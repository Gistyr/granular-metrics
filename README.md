# INSTALLATION
```toml
[dependencies]
granular-metrics = "0.1.0"
```
Or with optional [actix-web](https://crates.io/crates/actix-web) HTTP endpoint
```toml
[dependencies]
granular-metrics = { version = "0.1.0", features = ["http"] }
```
# HOW TO USE
## ONE: Define 
`#[derive(serde::Serialize)]` is only needed for the `http` feature                             
`#[derive(Debug, Clone, PartialEq, Eq, Hash)]` is mandatory
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
enum Keys {
    Login,
    Download,
    Ping,
    Verify,
}
```
- Choose `any` name for the `enum` type
- Each `variant` corresponds to a key and may be `named however you like`
- There’s `no limit` on how many `keys (variants)` you define
## TWO: Initialize
```rust
#[tokio::main]
async fn main() {
    granular_metrics::init::<Keys>();
}
```
Or with the [actix-web](https://crates.io/crates/actix-web) HTTP feature
```rust
#[tokio::main]
async fn main() {
    granular_metrics::init::<Keys>("127.0.0.1", "8080", "metrics", "all");
}
```
`pub fn init<K>(address: &str, port: &str, path: &str, workers: &str)`             
`http://127.0.0.1:8080/metrics`
- **address:** The IP address on which the HTTP server will listen
- **port:** The port number to listen on
- **path:** The URL segment `(without leading slash)` that is exposed
- **workers:** How many [actix-web](https://crates.io/crates/actix-web) `worker threads` to spawn
    - A value of `"all"` or `""` will spawn the default amount `(matches the number of CPU cores)`
    - Or give it a specific `number`, such as `"2"`
## THREE: INCREMENT
Call the `increment(Keys::Variant)` function to increase that key’s counter `by one`     
```rust
use crate::Keys::*;
use granular_metrics::increment;

async fn login_handler() -> impl Responder {
    increment(Login);
}

async fn download_handler() -> impl Responder {
    increment(Download);
}
```
## FOUR: Retrieve
```rust
async fn main() {
    let server_metrics = granular_metrics::fetch::<Keys>();
}
```
Or with the [actix-web](https://crates.io/crates/actix-web) HTTP feature
```
curl http://127.0.0.1:8080/metrics
```
```rust
// Make a custom implementation to retrieve and store your data
```
#### What you are retrieving:
```rust
pub struct MetricsSnapshot<K> {
    pub per_key: HashMap<K, (u64, u64)>,
    pub total: (u64, u64),
}
```

























