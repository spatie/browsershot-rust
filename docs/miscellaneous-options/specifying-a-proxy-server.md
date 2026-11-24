---
title: Specifying-a-proxy-server
weight: 19
---

You can specify a proxy server to use when connecting. The argument passed to `set_proxy_server` will be passed to the `--proxy-server=` option of Chromium. More info here: https://www.chromium.org/developers/design-documents/network-settings#TOC-Command-line-options-for-proxy-settings

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
   .set_proxy_server("1.2.3.4:8080");
# Ok(())
# }
```
