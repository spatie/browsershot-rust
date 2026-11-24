---
title: Using HTTP Authentication
weight: 23
---

You can provide credentials for HTTP authentication:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .authenticate("username", "password");
# Ok(())
# }
```
