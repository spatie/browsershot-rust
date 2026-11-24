---
title: Testing
weight: 5
---

Run these checks from the package directory:

Install Chrome or Chromium first. Set `CHROME_PATH` if the executable is not on your `PATH`. Integration tests launch a real browser and use local HTTP fixtures; there is no dependency on external websites. Set `RUST_TEST_THREADS=2` to limit concurrent browser processes on smaller machines.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
cargo doc --locked --no-deps --all-features
cargo package --locked
```

Set `RUSTDOCFLAGS` to `-D warnings` when running the documentation commands. In POSIX shells, prefix each command with `RUSTDOCFLAGS='-D warnings'`. In PowerShell, first run `$env:RUSTDOCFLAGS = '-D warnings'`.

README.md and all Markdown pages in docs/ participate in rustdoc tests. Setup lines prefixed with `# ` are hidden in rendered rustdoc. Browser examples are compiled with `no_run` so the examples keep their original URLs and output paths without making network requests or writing files during doctests. Integration tests execute browser behavior and assert the output bytes, pixels, page content and diagnostics.

Check the minimum supported Rust version:

```sh
rustup toolchain install 1.89.0 --profile minimal
cargo +1.89.0 test --locked --all-targets --all-features
cargo +1.89.0 test --locked --doc --all-features
```

Check security advisories:

```sh
cargo install cargo-audit --locked
cargo audit
```

CI runs stable checks on Linux, macOS and Windows, and the MSRV and audit checks on Linux. An audit requires access to the RustSec advisory database.
