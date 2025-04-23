# HOW TO USE
## ONE: Define 
```rust
// Must derive all 
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
enum Keys { // Can be named anything
    One,
    Two,
    Three,
    Four,
}
```
