# Bevy Efficient Forest Rendering

## Author: Pinkponk <henrik.djurestal@gmail.com>

This package, built in Rust language, contains all the code for the `bevy_efficient_forest_rendering` version 0.1.0. The main features of this package include terrain generation, procedural noise generation, and viewport utilization for efficient 3D rendering, especially useful for games and simulations.

The package includes various plugins like `OrbitCameraPlugin` and `ChunkGrassPlugin`, which facilitate camera control in a 3D environment and implements a system for rendering 3D grass atop terrain chunks.

The project also includes a `DistanceCulling` component which eliminates objects beyond a certain distance in the viewport and maintains smooth graphics rendering. Another aspect of this package includes the `Chunk` struct, representing individual segments in the game world, making it easier to manage world data in chunked form.

## Dependencies

Our project uses some dependencies that need to be installed:

- bevy v0.11.2
- bytemuck v1.11.0
- bevy_pbr v0.11.2
- itertools
- rand
- wasm-bindgen v0.2.84
- noise v0.8.2
- iyes_progress v0.9.1
- bevy_asset_loader v0.17.0
- bevy_web_fullscreen (specifically for targets based on `wasm32` architecture)

## Usage

### Build and Run

For a direct build and run use the following command:

```
cargo run --example forest --release --target wasm32-unknown-unknown
```

### Debugging

Two debugging profiles are provided for the Visual Studio Code editor, one for general execution and one for running unit tests. Use Visual Studio Code's built-in LLDB support to do debugging on your local machine.

## Contribution

Please before making any contribution to the project make sure to contact us through our email address: `henrik.djurestal@gmail.com`. 

Documentation contribution is as valuable as code contribution, please feel free to enhance, correct, or add to the documentation as you see fit.

Happy coding!

## License

This project is licensed under MIT License. For more information, see the LICENSE file in the repository.

## Disclaimer

The efficiencies gained from using this project depend on the specific hardware and software configuration. The author is not responsible for any potential loss or damage resulting from the use of this project. Always backup your work and test thoroughly before deploying.
