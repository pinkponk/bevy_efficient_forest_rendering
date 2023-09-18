mod framerate;
mod orbit;

use bevy::prelude::*;

pub use framerate::*;
pub use orbit::*;

pub struct HelpersPlugin;

impl Plugin for HelpersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DebugFrameratePlugin)
            .add_plugins(OrbitCameraPlugin);
    }
}
