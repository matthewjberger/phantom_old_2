mod animation;
mod camera;
mod gltf;
mod physics;
mod registry;
mod scenegraph;
mod texture;
mod transform;
mod world;

use phantom_dependencies::serde::{Deserialize, Serialize};

pub use self::{
    animation::*, camera::*, gltf::*, physics::*, registry::*, scenegraph::*, texture::*,
    transform::*, world::*,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Name(pub String);
