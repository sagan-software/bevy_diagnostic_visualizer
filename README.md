# `bevy_diagnostic_visualizer`

[![Crates.io](https://img.shields.io/crates/v/bevy_diagnostic_visualizer.svg)](https://crates.io/crates/bevy_diagnostic_visualizer)
[![Documentation](https://docs.rs/bevy_diagnostic_visualizer/badge.svg)](https://docs.rs/bevy_diagnostic_visualizer)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![Downloads](https://img.shields.io/crates/d/bevy_diagnostic_visualizer.svg)](https://crates.io/crates/bevy_diagnostic_visualizer)

This crate provides a plugin for visualizing Bevy game engine diagnostics.

## Usage

```rust
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_diagnostic_visualizer::DiagnosticVisualizerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(DiagnosticVisualizerPlugin::default())
        .run();
}
```

## License

Licensed under either of:

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

The [assets](assets) included in this repository typically fall under different
open licenses.  These will not be included in your game (unless copied in by you),
and they are not distributed in the published crate. See [CREDITS.md](CREDITS.md)
for the details of the licenses of those files.