---
title: Connection to a remote chromium/chrome instance
weight: 6
---

If you have a remote endpoint for a running chromium/chrome instance, properly configured with the param --remote-debugging-port, you can connect to it using the method `set_remote_instance`. You only need to specify its ip and port. If no instance is available at the given endpoint (instance crashed, restarting instance, etc), this will fallback to launching a chromium instance.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .set_remote_instance("1.2.3.4", 9222);
# Ok(())
# }
```

Use `set_ws_endpoint` if you already have the browser's `ws://` or `wss://` DevTools URL. To return an error instead of launching locally when the remote browser is unavailable, call `throw_on_remote_connection_error(true)`.

Browsershot closes only the page it creates on a remote instance, not the browser itself. Keep DevTools endpoints on a trusted network; access to one gives control of the browser.
