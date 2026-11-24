---
title: Using a pipe instead of a WebSocket
weight: 21
---

If you want to connect to the browser over a pipe instead of a WebSocket, you can use:

Pipes are supported for locally launched Chrome on Linux, macOS and Windows. Remote browsers always use a WebSocket.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .use_pipe();
# Ok(())
# }
```
