---
title: Setting the timeout
weight: 17
---

The default timeout of Browsershot is set to 60 seconds. Of course, you can modify this timeout:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .timeout(120)
    .save(path_to_image)?;
# Ok(())
# }
```

The timeout covers browser startup, navigation, waits and output. You can set the timeout for individual DevTools commands with `protocol_timeout`, also in seconds. The default protocol timeout is 30 seconds.
