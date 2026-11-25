# CivilCAD

## Run the desktop app

Install Rust with [rustup](https://rustup.rs/). On Ubuntu, install the graphics development libraries before building.

```sh
sudo apt-get install libfontconfig-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev libvulkan1
```

On Linux, the app needs a Vulkan driver. On Windows, install the Visual Studio Build Tools with the C++ desktop workload. On macOS, install the Xcode Command Line Tools with `xcode-select --install`.

```sh
cargo run --locked --release
```

To pan, drag with the middle mouse button or hold Space and drag in the focused drawing. Scroll or pinch to zoom around the pointer. Use **Reset view** to return to the origin. Drag the divider next to **Layers** to resize the panel.

## Build a GitHub release

1. Once the workflow is on the default branch, open **Actions → Release → Run workflow**.
2. Download the packages from the workflow run, or open the draft under **Releases** and publish it when ready.

The workflow builds Linux x86-64 on Ubuntu 24.04, macOS for Apple Silicon, and Windows x86-64. Extract the archive and open `civilcad`, `CivilCAD.app`, or `civilcad.exe`. The macOS build uses ad-hoc signing. The macOS build is not notarized, and the Windows executable is unsigned. Your OS may show a security prompt when you open a downloaded build.

The release tag comes from the version in `Cargo.toml`. Bump that version and update `Cargo.lock` before creating the next release.
