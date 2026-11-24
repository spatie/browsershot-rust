---
title: Adding extra headers to every request
weight: 1
---

To add custom HTTP headers to the navigational HTTP request and all resources that make up the page, use `set_extra_http_headers`:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .set_extra_http_headers([("Custom-Header-Name", "Custom-Header-Value")]);
# Ok(())
# }
```
