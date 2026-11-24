---
title: Read redirect history
weight: 26
---

Sometimes it can happen that the screenshot made by Browsershot is different from the one displayed by your browser.
It is often due to one or more redirects made by the site, based on geolocation, cookies, etc...

To understand what happens, you can keep track of the redirects made by the site, both HTTP, Javascript and HTML ones.
In addition to the URLs there are also available HTTP status, reason and headers.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let redirects = Browsershot::url("https://www.spatie.be")
    .redirect_history()?;

for redirect in redirects {
    println!("{}", redirect["url"]);
    println!("{}", redirect["status"]);
    println!("{}", redirect["reason"]);
    for (name, value) in redirect["headers"].as_object().into_iter().flatten() {
        println!("{name} : {value}");
    }
    println!();
}
# Ok(())
# }
```
