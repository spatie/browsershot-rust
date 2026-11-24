---
title: Fixing cors issues
weight: 8
---

If you experience issues related to [cors](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS), you can opt to disable cors checks with [--disable-web-security](https://peter.sh/experiments/chromium-command-line-switches/#disable-web-security).

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
   .add_chromium_arguments(["disable-web-security"])
   .save(path_to_image)?;
# Ok(())
# }
```
