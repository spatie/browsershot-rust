---
title: Requirements
weight: 3
---

This package requires Rust 1.89 or higher and an installed version of Google Chrome or Chromium. Chrome is not bundled with the package. Node and Puppeteer are not required.

### Installing Chrome on macOS

On macOS you can install Chrome via Homebrew:

```bash
brew install --cask google-chrome
```

### Installing Chromium on Ubuntu

On Ubuntu you can install Chromium like this:

```bash
sudo apt update
sudo apt install chromium-browser
```

On Ubuntu this may install Chromium through Snap. In containers, use a Chrome or Chromium installation that supports your container environment. Keep Chrome's sandbox enabled whenever possible.

### Custom chrome/chromium executable path

If you want to use an alternative chrome or chromium executable you can set it using the `set_chrome_path` method.

```rust,no_run
# use spatie_browsershot::Browsershot;
# let _ =
Browsershot::html("Foo")
  .set_chrome_path("/path/to/my/chrome");
```

By default, Browsershot checks `CHROME_PATH`, then the system `PATH` and the standard Chrome installation paths on macOS and Windows.

### Pass custom arguments to Chromium

If you need to pass custom arguments to Chromium, use the `add_chromium_arguments` method.

The method accepts an iterator of arguments. All of these arguments will automatically be prefixed with `--`. Arguments with values use `key=value`.

```rust,no_run
# use spatie_browsershot::Browsershot;
# let _ =
Browsershot::html("Foo")
  .add_chromium_arguments([
      "some-argument-without-a-value",
      "keyed-argument=argument-value",
  ]);
```

| Example array | Flags that will be passed to Chromium |
| - | - |
| `["foo"]` | `--foo` |
| `["foo", "bar"]` | `--foo --bar` |
| `["foo", "bar=baz"]` | `--foo --bar=baz` |

This method can be useful in order to pass a flag to fix font rendering issues on some Linux distributions (e.g. CentOS).

```rust,no_run
# use spatie_browsershot::Browsershot;
# let _ =
Browsershot::html("Foo")
  .add_chromium_arguments([
      "font-render-hinting=none",
  ]);
```
