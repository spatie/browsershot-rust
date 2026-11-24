---
title: Setting the user agent
weight: 18
---

If you want to set the user agent Google Chrome should use when taking the screenshot you can do so:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .user_agent("My Special Snowflake Browser 1.0")
    .save(path_to_image)?;
# Ok(())
# }
```
