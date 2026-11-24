---
title: Evaluating a script before page load
weight: 27
---

Sometimes you need JavaScript to run before any of the page's own scripts execute. The `evaluate_on_new_document` method injects a script that runs before the page loads, on every page Browsershot navigates to. Internally it uses Chrome's [`Page.addScriptToEvaluateOnNewDocument`](https://chromedevtools.github.io/devtools-protocol/tot/Page/#method-addScriptToEvaluateOnNewDocument).

This is handy for stubbing globals, injecting polyfills, overriding `navigator.webdriver`, or disabling analytics before navigation.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .evaluate_on_new_document("window.myFlag = true")
    .save(path_to_image)?;
# Ok(())
# }
```

The argument is evaluated as a script, so pass a statement rather than a function expression. The script above sets `window.myFlag` before any of the page's own scripts run.
