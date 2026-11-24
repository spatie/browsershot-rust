---
title: Creating HTML
weight: 3
---


Browsershot also can get the body of an HTML page after JavaScript has been executed:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
Browsershot::url("https://example.com").body_html()?; // returns the html of the body
# Ok(())
# }
```

### Evaluate

Browsershot can get the evaluation of an html page:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
Browsershot::url("https://example.com")
  .device_scale_factor(2)
  .evaluate("window.devicePixelRatio")?; // returns 2
# Ok(())
# }
```
