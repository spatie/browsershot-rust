---
title: Passing environment variables to the browser
weight: 12
---

If you want to set custom environment variables which affect the browser instance you can use:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .set_environment_options([("TZ", "Pacific/Auckland")]);
# Ok(())
# }
```
