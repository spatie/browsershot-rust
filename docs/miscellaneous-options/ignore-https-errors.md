---
title: Ignore HTTPS errors
weight: 11
---

You can ignore HTTPS errors, if necessary.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .ignore_https_errors();
# Ok(())
# }
```
