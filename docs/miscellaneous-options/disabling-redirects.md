---
title: Disabling redirects
weight: 26
---

To avoid redirects to domains that are not allowed in your environment, or for security reasons you can disable HTTP redirects.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("http://www.spatie.be")
   .disable_redirects();
# Ok(())
# }
```
