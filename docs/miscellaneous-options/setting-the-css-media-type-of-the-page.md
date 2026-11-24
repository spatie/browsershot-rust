---
title: Setting the CSS media type of the page
weight: 16
---

You can emulate the media type, especially useful when you're generating pdf shots, because it will try to emulate the print version of the page by default.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_pdf = "example.pdf";
Browsershot::url("https://example.com")
    .emulate_media(Some("screen")) // Some("screen"), Some("print") (default) or None (disables the emulation).
    .save_pdf(path_to_pdf)?;
# Ok(())
# }
```

You can also emulate [media features](https://www.w3.org/TR/mediaqueries-5/), such as dark mode or reduced motion.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .emulate_media_features(json!([
        {"name": "prefers-color-scheme", "value": "dark"}
    ]))
    .save(path_to_image)?;
# Ok(())
# }
```
