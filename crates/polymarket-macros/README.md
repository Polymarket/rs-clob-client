# polymarket-macros

Procedural macros for the Polymarket client SDK.

## `#[unknown_enum_variant]`

Automatically adds an `Unknown(String)` variant to an enum and implements the `UnknownEnumVariant` trait for forward-compatible API deserialization.

### Usage

```rust
use polymarket_client_sdk::unknown_enum_variant;

#[unknown_enum_variant]
#[derive(Debug, Deserialize)]
enum OrderType {
    GTC,
    FOK,
}
```

This expands to:

```rust
#[derive(Debug, Deserialize)]
enum OrderType {
    GTC,
    FOK,
    #[serde(untagged)]
    Unknown(String),
}

impl UnknownEnumVariant for OrderType {
    fn as_unknown(&self) -> Option<&str> {
        match self {
            OrderType::Unknown(s) => Some(s),
            _ => None,
        }
    }

    fn type_name() -> &'static str {
        "OrderType"
    }
}
```
