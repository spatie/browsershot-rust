---
title: Disable sandboxing
weight: 7
---

When running Linux in certain virtualization environments it might need to disable sandboxing.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .no_sandbox();
# Ok(())
# }
```
