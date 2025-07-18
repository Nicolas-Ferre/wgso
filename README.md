# WGSO

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Nicolas-Ferre/wgso#license)
[![CI](https://github.com/Nicolas-Ferre/wgso/actions/workflows/ci.yml/badge.svg)](https://github.com/Nicolas-Ferre/wgso/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/Nicolas-Ferre/wgso)](https://app.codecov.io/gh/Nicolas-Ferre/wgso)

WGSO is a WebGPU Shader Orchestrator that can be used to create GPU-native applications written
exclusively in [WGSL](https://www.w3.org/TR/WGSL/).

It is particularly well suited for graphics applications such as games.

## ⚠️ Warning

Before you consider using this tool, please keep in mind that:

- It is developed by a single person in his spare time.
- The library is highly experimental, so it shouldn't be used for production applications.

## 🚀 Main features

- 🗒 Orchestrate execution of shaders
- ⚙️ Automate shader CPU side configuration
- 🔥 Maximize execution on GPU side
- 🔄 Hot reloadable

## 💻 Supported platforms

- Windows
- Linux
- macOS (limited support because the maintainer doesn't have access to a physical device)
- Android
- Web (browser should support WebGPU)

WGSO may also work on some other platforms, but they have not been tested.

## 🏁 Getting started

Examples of WGSO programs are located in the `examples` folder.

Example dependencies can be installed with the following command:

```shell
cargo run --release --bin wgso -- install --force <example path>
```

Then the example can be run with the following commands:

- Native platforms (Window, Linux and macOS):
    - ```shell
      cargo run --release --bin wgso -- run <example path>
      ```
- Android:
    - ```shell
      PROGRAM_PATH=<example absolute path> cargo apk run --manifest-path=crates/wgso/Cargo.toml --example wgso_android --release
      ```
- Web (browser should support WebGPU):
    - ```shell
      PROGRAM_PATH=<example absolute path> cargo run-wasm --example wgso_web --release
      ```

## 💥 Known issues

- Android: structs can sometimes have alignment issues which cause incorrect read of fields. Adding
  placeholder fields should resolve the problem. The issue has been found when a `f32` field is
  after a `vec3f` field:

```wgsl
struct MyObject {
    position: vec3f,
    _phantom: vec2f, // needed to make sure my_object.size and my_object.state return correct values
    size: f32,
    state: u32,
}
```

- Android: passing a mat4x4f as argument of a function may cause a segment fault. Passing the matrix
  as an `array<vec4f, 4>` argument or 4 `vec4f` arguments should fix the problem.

## 📜 License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### 🤝 Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or
conditions.
