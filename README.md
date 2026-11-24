# spatie-browsershot

[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/browsershot-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/browsershot-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/browsershot-rust/blob/main/LICENSE.md)

Convert HTML to an image, PDF or string. Browsershot controls Chrome directly over the DevTools protocol. No Node or Puppeteer installation is needed.

## Support us

[<img src="https://spatie.be/github-ad-image/browsershot-rust.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/browsershot-rust)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer and an installed Chrome or Chromium browser. See the [requirements](https://spatie.be/docs/browsershot-rust/v1/requirements) for installation and custom executable paths.

The crate is not published on crates.io. Use a Git dependency:

```toml
[dependencies]
spatie-browsershot = { git = "https://github.com/spatie/browsershot-rust", branch = "main" }
```

## Usage

```rust,no_run
use spatie_browsershot::Browsershot;
# fn main() -> Result<(), spatie_browsershot::Error> {

Browsershot::url("https://example.com").save("example.png")?;
Browsershot::html("<h1>Hello world!!</h1>").save("example.pdf")?;
# Ok(())
# }
```

Only pass URLs and HTML that you trust. Browsershot is not a security boundary for untrusted content.

See the [v1 documentation](https://spatie.be/docs/browsershot-rust/v1/introduction).

## Testing

Install Chrome and, if it is not on your `PATH`, set `CHROME_PATH` to its executable path. Tests use local fixtures and do not require external websites.

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

See [all quality checks](https://github.com/spatie/browsershot-rust/blob/main/docs/testing.md). The README and Markdown guides are tested by rustdoc, including setup on hidden `# ` lines.

## Changelog

See [CHANGELOG.md](https://github.com/spatie/browsershot-rust/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/browsershot-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/browsershot-rust/blob/main/SECURITY.md).

## Credits

- [Freek Van der Herten](https://github.com/freekmurze)
- [All contributors](https://github.com/spatie/browsershot-rust/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/spatie/browsershot-rust/blob/main/LICENSE.md).
