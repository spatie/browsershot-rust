---
title: Installation & setup
weight: 4
---

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
spatie-browsershot = "1"
```

This package requires Chrome or Chromium to be installed. You can find installation instructions on [the requirements page](/docs/browsershot-rust/v1/requirements).

### Using Browsershot on AWS Lambda

Provide a Chrome executable built for your Lambda runtime and architecture, and pass its path to `set_chrome_path`. The crate does not install a browser or a Lambda layer for you. Put writable profiles and temporary files in Lambda's temporary directory using `set_custom_temp_path`.
