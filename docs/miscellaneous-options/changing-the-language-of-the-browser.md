---
title: Changing the language of the browser
weight: 3
---

You can use `set_environment_options` to change the language of the browser.
In order to load a page in a specific language for example.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .set_environment_options([("LANG", "en-GB")]);
# Ok(())
# }
```
