use bevy::prelude::*;

use crate::setup_shadow_pass_system;
use crate::shadow_caster::spawn_shadows_system;
use crate::shadow_pass::ShadowMap;

pub struct ShadowPlugin;

impl Plugin for ShadowPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ShadowMap(None))
            .add_system(spawn_shadows_system)
            .add_startup_system(setup_shadow_pass_system);
    }
}
