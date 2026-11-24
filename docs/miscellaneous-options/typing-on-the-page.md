---
title: Typing on the page
weight: 20
---

You can type on the page (you can use this to fill form fields).

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .r#type("#selector1", "Hello, is it me you are looking for?");
# Ok(())
# }
```

You can combine `type` and `click` to create a screenshot of a page after submitting a form:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
# let milliseconds_to_wait = 1000;
Browsershot::url("https://example.com")
    .r#type("#firstName", "My name")
    .click("#submit")
    .delay(milliseconds_to_wait)
    .save(path_to_image)?;
# Ok(())
# }
```
