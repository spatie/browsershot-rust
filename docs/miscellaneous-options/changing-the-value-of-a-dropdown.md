---
title: Changing the value of a dropdown
weight: 4
---

You can change the value of a dropdown on the page (you can use this to change form select fields).

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::url("https://example.com")
    .select_option("#selector1", "100");
# Ok(())
# }
```

You can combine `select_option`, `type` and `click` to create a screenshot of a page after submitting a form:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
# let milliseconds_to_wait = 1000;
Browsershot::url("https://example.com")
    .r#type("#firstName", "My name")
    .select_option("#state", "MT")
    .click("#submit")
    .delay(milliseconds_to_wait)
    .save(path_to_image)?;
# Ok(())
# }
```
