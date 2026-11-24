---
title: Adding extra headers to the navigational request
weight: 2
---

To add custom HTTP headers to a navigational HTTP request, use `extraNavigationHTTPHeaders` like so:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .set_extra_navigation_http_headers([("Custom-Header-Name", "Custom-Header-Value")]);
# Ok(())
# }
```

This will add the header to the page you want to render, but those headers will not be added to any external resources that make up that page.
