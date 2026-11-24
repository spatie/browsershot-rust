---
title: Getting failed requests
weight: 10
---

To get all failed requests encountered while loading a page call `failed_requests`.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let console_messages = Browsershot::url("https://example.com").failed_requests()?; // returns an array
# Ok(())
# }
```

The `failed_requests` method return an array of which each item is yet another array with these keys. Image that a page contains a broken image

- `status`: the status code, for a broken image: `404`
- `url`: the url of the broken image: `https://example.com/image.jpg`
