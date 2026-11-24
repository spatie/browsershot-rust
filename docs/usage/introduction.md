---
title: Introduction
weight: 1
---

In all examples it is assumed that you imported this type at the top of your file

```rust,no_run
use spatie_browsershot::Browsershot;
```

Browsershot can read the HTML of a URL and do something with it, for example convert it to pdf

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
Browsershot::url("https://example.com").save("example.pdf")?;
# Ok(())
# }
```

Alternatively, you can  use an arbitrary html input, using the `html` function:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
Browsershot::html("<h1>Hello world!!</h1>").save("example.pdf")?;
# Ok(())
# }
```

Here are the things that Browsershot can produce for you:

- [an image](/docs/browsershot-rust/v1/usage/creating-images)
- [a PDF](/docs/browsershot-rust/v1/usage/creating-pdfs)
- [HTML](/docs/browsershot-rust/v1/usage/creating-html)

Output methods return a `Result`. Use `?` to propagate errors. Examples use hidden setup lines for imports and input variables; each example is compiled by rustdoc. Examples that navigate or write output are not run by rustdoc.

Builders consume and return themselves. Output methods borrow mutably, so you can save a screenshot and then read diagnostics from the same operation. The methods `triggered_requests`, `console_messages`, `failed_requests`, `redirect_history` and `page_errors` reuse the last operation's diagnostics when available. `output()` gives borrowed access to all of them without copying.

The output also contains `result` and `exception`. Evaluation results keep their JSON types, HTML output is a string, and binary browser output is base64-encoded. An operation error remains available in `exception` alongside the captured diagnostics.

## Local file access

HTML passed to `html()` or `set_html()` cannot load local files by default. Browsershot blocks `file:` requests in Chrome, including requests from images, stylesheets, scripts and iframes. This also blocks absolute filesystem paths such as `/var/www/app/public/logo.png`, which resolve to local files when rendering HTML without a content URL, and file URLs constructed by JavaScript.

Blocked resources will not appear in the output. File URLs in HTML are blocked during rendering rather than rejected by `html()` with an exception.

HTML containing `view-source` URLs or schemeless local network paths such as `//localhost/file` and `\\localhost\file` is rejected when rendering, even if local file access is enabled. Explicit HTTP(S) URLs are still allowed.

If your trusted HTML intentionally loads local assets, you can explicitly allow local file access:

```rust,no_run
# use spatie_browsershot::{Browsershot, ClickOptions, Polling};
# use std::time::Duration;
# use serde_json::json;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
Browsershot::html("<img src=\"file:///var/www/app/public/logo.png\">")
    .allow_local_file_access(true)
    .save("example.pdf")?;
# Ok(())
# }
```

This applies to PDFs, screenshots and HTML evaluation. You can restore the default restriction with `allow_local_file_access(false)`.

Only enable this for HTML you trust. It permits access to any local file readable by Chrome, not just assets in your application's public directory.

`html_from_file_path()` and `set_html_from_file_path()` intentionally render local HTML files and retain their existing local-resource access. `set_content_url()` still controls how relative paths are resolved, and HTTP(S) resources are not blocked by the local-file restriction. Allowing local file access does not override Chrome's own security restrictions.

Browsershot is not a sandbox for untrusted HTML. You are still responsible for validating HTML and URLs. If you render untrusted content, isolate the browser's filesystem and network access; blocking local files does not prevent requests to internal HTTP services.
