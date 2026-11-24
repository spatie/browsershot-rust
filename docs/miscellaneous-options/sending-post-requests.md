---
title: Sending POST requests
weight: 14
---

By default, all requests sent using GET method. You can make POST request to the given url by using the `post` method.
Note: POST request sent using `application/x-www-form-urlencoded` content type.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .post([("foo", "bar")]);
# Ok(())
# }
```
