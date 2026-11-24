---
title: Getting console output
weight: 9
---

To get all output of the Chrome console call `console_messages`.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let console_messages = Browsershot::url("https://example.com").console_messages()?; // returns an array
# Ok(())
# }
```

The `console_messages` method return an array of which each item is yet another array with these keys:

- `type`: the type of output (`log`, `error`, ...)
- `message`: the message itself
- `location`: an array with information on where the console message was triggered. In most cases, the `url` key will contain the URL where the message was triggered.
