---
title: Clicking on the page
weight: 5
---

You can specify clicks on the page.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let right = 10.0;
# let _ =
Browsershot::url("https://example.com")
    .click("#selector1")
    // Right click 5 times on #selector2, each click lasting 200 milliseconds.
    .click_with_options("#selector2", ClickOptions {
        button: "right".into(),
        click_count: 5,
        delay: Duration::from_millis(200),
    });
# Ok(())
# }
```
