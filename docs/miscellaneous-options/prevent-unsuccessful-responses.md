---
title: Prevent unsuccessful responses
weight: 13
---

You may want to throw an error when the page response is unsuccessful, you can use the following method :

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .prevent_unsuccessful_response(true);
# Ok(())
# }
```
