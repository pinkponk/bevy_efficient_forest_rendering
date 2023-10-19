use bevy::prelude::*;

pub mod chunk_grass; // Importing a module named chunk_grass

pub mod chunk_instancing; // Importing a module named chunk_instancing

#[derive(Component, Debug)] // Defining a struct named DistanceCulling that has the Component and Debug traits
pub struct DistanceCulling {
    pub distance: f32, // A public field named distance of type f32
}

impl Default for DistanceCulling {
    fn default() -> Self {
        Self { distance: 1000.0 } // Implementing the default method for DistanceCulling that returns a new instance with a default distance value of 1000.0
    }
}

#[derive(Component, Default, Debug, Clone)] // Defining a struct named Chunk that has the Component, Default, Debug, and Clone traits
pub struct Chunk {
    pub chunk_xy: [u32; 2], // A public field named chunk_xy of type [u32; 2]
}