---
title: Using url for html content
weight: 24
---

Using the method *set_content_url* you can set the base url of the request when using the *html* method. Sometimes you may have relative paths in your code. When passing a html page to Chrome, you don't have a base url set. So any relative path present in your html content will not fetch correctly.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let _ =
Browsershot::html("<html>... relative paths to fetch ...</html>")
   .set_content_url("https://www.example.com");
# Ok(())
# }
```
