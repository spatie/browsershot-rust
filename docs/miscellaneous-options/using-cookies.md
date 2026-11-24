---
title: Using cookies
weight: 22
---

You can add cookies to the request to the given url:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .use_cookies([("Cookie-Key", "Cookie-Value")], None);
# Ok(())
# }
```

You can specify the domain to register cookies to, if necessary:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .use_cookies([("Cookie-Key", "Cookie-Value")], Some("ui.example.com"));
# Ok(())
# }
```
