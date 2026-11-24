---
title: Creating images
weight: 2
---

Here's the easiest way to create an image of a webpage:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com").save(path_to_image)?;
# Ok(())
# }
```

## Formatting the image

By default, the screenshot's type will be a `png`.
But you can change it to `jpeg` with quality option.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .set_screenshot_type("jpeg", Some(100))
    .save(path_to_image)?;
# Ok(())
# }
```

## Sizing the image

By default the screenshot's size is 800 by 600 pixels. Want another size of screenshot? No problem!

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .window_size(640, 480)
    .save(path_to_image)?;
# Ok(())
# }
```

You can screenshot only a portion of the page by using `clip`.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
# let x = 0;
# let y = 0;
# let width = 640;
# let height = 480;
Browsershot::url("https://example.com")
    .clip(x, y, width, height)
    .save(path_to_image)?;
# Ok(())
# }
```

You can take a screenshot of an element matching a selector using `select` and a `selector_index` which is used to select the nth element (e.g. use `selector_index = 3` to get the fourth element like `div:eq(3)`). Pass `0` for the first matching element.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
# let selector_index = 0_usize;
Browsershot::url("https://example.com")
    .select(".some-selector", selector_index)
    .save(path_to_image)?;
# Ok(())
# }
```

### Getting a screenshot as base64

If you need the base64 version of a screenshot you can use the `base64_screenshot` method. This can come in handy when you don't want to save the screenshot on disk.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let base64_data = Browsershot::url("https://example.com")
    .base64_screenshot()?;
# Ok(())
# }
```

## Manipulating the image

Image manipulation uses Rust's [image](https://docs.rs/image) crate. Here's an example where we create a greyscale image:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .window_size(640, 480)
    .greyscale()
    .save(path_to_image)?;
# Ok(())
# }
```

For other transformations, pass an image transformation function to `manipulate`. It receives and returns an `image::DynamicImage`. Transformations run in order and apply to both file and in-memory screenshots.

## Taking a full page screenshot

You can take a screenshot of the full length of the page by using `full_page()`.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .full_page()
    .save(path_to_image)?;
# Ok(())
# }
```

## Setting the device scale

You can also capture the webpage at higher pixel densities by passing a device scale factor value of 2 or 3. This mimics how the webpage would be displayed on a retina/xhdpi display.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .device_scale_factor(2)
    .save(path_to_image)?;
# Ok(())
# }
```

## Mobile emulation

You can emulate a mobile view with the `mobile` and `touch` methods.
`mobile` will set the display to take into account the page's meta viewport, as Chrome mobile would.
`touch` will set the browser to emulate touch functionality, hence allowing spoofing for pages that check for touch.
Along with the `user_agent` method, these can be used to effectively take a mobile screenshot of the page.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .user_agent("My Mobile Browser 1.0")
    .mobile()
    .touch()
    .save(path_to_image)?;
# Ok(())
# }
```

## Device emulation

You can emulate a device view with the `device` method. The devices' names can be found [Here](https://github.com/puppeteer/puppeteer/blob/main/docs/api/puppeteer.knowndevices.md).

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
let browsershot = Browsershot::url("https://example.com");
browsershot
        .device("iPhone X")
        .save(path_to_image)?;
# Ok(())
# }
```

is the same as

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 11_0 like Mac OS X) AppleWebKit/604.1.38 (KHTML, like Gecko) Version/11.0 Mobile/15A372 Safari/604.1")
    .window_size(375, 812)
    .device_scale_factor(3)
    .mobile()
    .touch()
    .landscape(false)
    .save(path_to_image)?;
# Ok(())
# }
```

## Backgrounds

If you want to ignore the website's background when capturing a screenshot, use the `hide_background()` method.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .hide_background()
    .save(path_to_image)?;
# Ok(())
# }
```

## Dismiss dialogs

Javascript pop ups such as alerts, prompts and confirmations cause rendering of the site to stop, which leads to an empty screenshot. Calling `dismiss_dialogs()` method automatically closes such popups allowing the  screenshot to be taken.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .dismiss_dialogs()
    .save(path_to_image)?;
# Ok(())
# }
```

## Disable Javascript

If you want to completely disable javascript when capturing the page, use the `disable_javascript()` method.
Be aware that some sites will not render correctly without javascript.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .disable_javascript()
    .save(path_to_image)?;
# Ok(())
# }
```

## Disable Images

You can completely remove all images and <img> elements when capturing a page using the `disable_images()` method.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .disable_images()
    .save(path_to_image)?;
# Ok(())
# }
```

## Block Urls

You can completely block connections to specific Urls using the `block_urls()` method.
Useful to block advertisements and trackers to make screenshot creation faster.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
let urls_list = ["example.com/cm-notify?pi=outbrain", "sync.outbrain.com/cookie-sync?p=bidswitch"];
Browsershot::url("https://example.com")
    .block_urls(urls_list)
    .save(path_to_image)?;
# Ok(())
# }
```

## Block Domains

You can completely block connections to specific domains using the `block_domains()` method.
Useful to block advertisements and trackers to make screenshot creation faster.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
let domains_list = ["googletagmanager.com", "googlesyndication.com", "doubleclick.net", "google-analytics.com"];
Browsershot::url("https://example.com")
    .block_domains(domains_list)
    .save(path_to_image)?;
# Ok(())
# }
```

## Waiting for lazy-loaded resources

Some websites lazy-load additional resources via ajax or use webfonts, which might not be loaded in time for the screenshot. Using the `wait_until_network_idle(true)` method you can tell Browsershot to wait for a period of 500 ms with no network activity before taking the screenshot, ensuring all additional resources are loaded.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .wait_until_network_idle(true)
    .save(path_to_image)?;
# Ok(())
# }
```

Alternatively you can use less strict `wait_until_network_idle(false)`, which allows 2 network connections in the 500 ms waiting period, useful for websites with scripts periodically pinging an ajax endpoint.

## Delayed screenshots

You can delay the taking of screenshot by  `set_delay()`. This is useful if you need to wait for completion of javascript or if you are attempting to capture lazy-loaded resources.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
# let delay_in_milliseconds = 1000;
Browsershot::url("https://example.com")
    .set_delay(delay_in_milliseconds)
    .save(path_to_image)?;
# Ok(())
# }
```

## Waiting for javascript function

You can also wait for a javascript function until is returns true by using `wait_for_function()`. This is useful if you need to wait for task on javascript which is not related to network status.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
# let polling_in_milliseconds = 100;
# let timeout_in_milliseconds = 10000;
Browsershot::url("https://example.com")
    .wait_for_function("window.innerWidth < 100", Polling::Interval(Duration::from_millis(polling_in_milliseconds)), Duration::from_millis(timeout_in_milliseconds))
    .save(path_to_image)?;
# Ok(())
# }
```

## Waiting for a selector

You can also wait for a selector by using `wait_for_selector()`. This is useful if you need to wait for the selector to appear in page.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .wait_for_selector("#my_selector")
    .save(path_to_image)?;
# Ok(())
# }
```

## Adding JS

You can add javascript prior to your screenshot or output using the syntax for [Puppeteer's addScriptTag](https://github.com/puppeteer/puppeteer/blob/v1.9.0/docs/api.md#pageaddscripttagoptions).

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .set_option("addScriptTag", json!({"content" : "alert(\"Hello World\")"}))
    .save(path_to_image)?;
# Ok(())
# }
```

## Adding CSS

You can add CSS styles prior to your screenshot or output using the syntax for [Puppeteer's addStyleTag](https://github.com/puppeteer/puppeteer/blob/v1.9.0/docs/api.md#pageaddstyletagoptions).

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let path_to_image = "example.png";
Browsershot::url("https://example.com")
    .set_option("addStyleTag", json!({"content" : "body{ font-size: 14px; }"}))
    .save(path_to_image)?;
# Ok(())
# }
```

## Output directly to the browser

You can output the image directly to the browser using the `screenshot()` method.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let image = Browsershot::url("https://example.com").screenshot()?;
# Ok(())
# }
```

## Setting the user data directory

You can set the [user data directory](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/user_data_dir.md) that is used to store the browser session and additional data. Setting this to a static value may introduce cache problems, could also increase performance. It needs to be an absolute path.

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let image = Browsershot::url("https://example.com")
    .user_data_dir("/tmp/session-1")
    .screenshot()?;
# Ok(())
# }
```
