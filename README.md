# WGSO

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Nicolas-Ferre/wgso#license)
[![CI](https://github.com/Nicolas-Ferre/wgso/actions/workflows/ci.yml/badge.svg)](https://github.com/Nicolas-Ferre/wgso/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/Nicolas-Ferre/wgso)](https://app.codecov.io/gh/Nicolas-Ferre/wgso)

WGSO is a WebGPU Shader Orchestrator that can be used to create GPU-native applications written
exclusively in [WGSL](https://www.w3.org/TR/WGSL/).

It is particularly well suited for graphics applications such as games.

## ⚠️ Warning ⚠️

Before you consider using this tool, please keep in mind that:

- It is developed by a single person in his spare time.
- The library is highly experimental, so it shouldn't be used for production applications.

## Supported platforms

- Windows
- Linux
- macOS (limited support because the maintainer doesn't have access to a physical device)

It is planned to also support Android and WASM targets in the future.

WGSO may also work on some other platforms, but they have not been tested.

## Getting started

WGSO examples can be run with the following command:

```shell
cargo run --release --bin wgso -- run <example path>
```

Examples of WGSO programs are located in the `examples` folder.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or
conditions.
