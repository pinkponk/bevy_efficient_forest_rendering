# bevy_efficient_forest_rendering

![logo](logo.png)

![Rust Version](https://img.shields.io/badge/Rust-2021-ed1c24) 
![Bevy Engine](https://img.shields.io/badge/Bevy-0.11.2-blueviolet)

## Description

`bevy_efficient_forest_rendering` is a dedicated solution for creating visually impactful and performance-optimized 3D forest environments. Authored by Pinkponk, this software package utilizes the Bevy game engine and is entirely written in the Rust programming language. It employs cutting-edge techniques to manage layers of visual complexity and ensure fast, smooth rendering, even on a web platform.

This package uses different sections to handle various aspects of the environment, such as ground and texture creation, grass rendering, vertex manipulation, and orbital camera creation. The code includes a myriad of variables and functions for random generation, allowing design elements within the environment to exhibit natural variation. The package also contains numerous structures, render commands, and traits that are essential for gaming environment set up. 

The `bevy_efficient_forest_rendering` package can render an impressive 8 million grass straws at 60fps on a standard gaming PC setup, providing excellent performance metrics and capabilities. The grass rendering system has been highly optimized to provide 3x-4x better frames per second than the more general GPU instancing.

In addition, this package provides robust debugging and testing support. It is also compatible with WebAssembly environments, making it extremely flexible and versatile for cross-platform web games development.

## Installation Procedures
To install `bevy_efficient_forest_rendering`, make sure you have Rust 2021 edition installed on your machine. Afterward, simply clone the project and install the necessary dependencies as specified in the `Cargo.toml` file. 

```bash
git clone https://github.com/pinkponk/bevy_efficient_forest_rendering.git
cd bevy_efficient_forest_rendering
cargo build --release
```

## Usage Instructions

To run the example 'forest', use the following command:

```bash
cargo run --example forest --release --target wasm32-unknown-unknown
```
For debugging, one should refer to the provided `.vscode/launch.json` file.

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug executable 'bevy_efficient_forest_rendering'",
     "cargo": {
         "args": [
             "build",
             "example",
             "forest"
        ],
     },
     "args": [],
     "cwd": "${workspaceFolder}",
     "env": {
         "RUST_LOG": "warn,bevy_efficient_forest_rendering=debug",
         "CARGO_MANIFEST_DIR": "${workspaceFolder}",
     },
}
```

> Note: Make sure you've installed the necessary debuggers and the runner as specified in the `.cargo/config.toml`.

For a more illustrative understanding of the project, please follow the development logs available [here](https://www.youtube.com/channel/UCqzbiRaNXJa50J4kvJvKpAg).

## Contributing

Contributions are more than welcome! Simply fork the repository, make your changes and then submit a pull request.

## License

This project is licensed under the MIT license.

For any issues or suggestions, please reach out to the author at `henrik.djurestal@gmail.com`.
