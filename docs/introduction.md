---
title: Introduction
weight: 1
---

The package can convert a webpage to an image or pdf. The conversion is done behind the scenes by controlling a headless version of Google Chrome through the [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/). Node and Puppeteer are not needed.

Here's a quick example:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
// an image will be saved
Browsershot::url("https://example.com").save(path_to_image)?;
# Ok(())
# }
```

It will save a pdf if the path passed to the `save` method has a `pdf` extension.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
// a pdf will be saved
Browsershot::url("https://example.com").save("example.pdf")?;
# Ok(())
# }
```

You can also use an arbitrary html input, simply replace the `url` method with `html`:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
Browsershot::html("<h1>Hello world!!</h1>").save("example.pdf")?;
# Ok(())
# }
```

Browsershot also can get the body of an HTML page after JavaScript has been executed:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
Browsershot::url("https://example.com").body_html()?; // returns the html of the body
# Ok(())
# }
```

If you wish to retrieve an array list with all the requests that the page triggered you can do so:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let requests = Browsershot::url("https://example.com")
    .triggered_requests()?;

for request in requests {
    let url = &request["url"]; // https://example.com/
}
# Ok(())
# }
```
