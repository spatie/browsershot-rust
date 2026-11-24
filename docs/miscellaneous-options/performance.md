---
title: Performance
weight: 13
---

The default behavior of the browsershot is to capture requested addresses.
So when you use [data urls](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs) on the HTML document , this capturing is not good for memory because more addresses consume more memory.

You can disable this option by calling `disable_capture_urls()`.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::html("More data urls exists in this document")
    .disable_capture_urls()
;
# Ok(())
# }
```