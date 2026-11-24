---
title: Setting an arbitrary option
weight: 15
---

You can set any arbitrary options by calling `set_option`:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
   .set_option("landscape", true)
   .save(path_to_image)?;
# Ok(())
# }
```

Options use the browser option names shown above. Dotted keys such as `viewport.width` are supported. Unknown options return `Error::InvalidInput` when rendering rather than being silently ignored. There is no Puppeteer process to receive Puppeteer-specific options.
