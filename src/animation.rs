use bevy::prelude::*;

use crate::state::GameState;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Play).with_system(update_animations));
    }
}

#[derive(Component)]
pub struct Animation {
    pub running: bool,
    pub timer: Timer,
}

fn update_animations(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut Animation,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut anim, mut sprite, texture_atlas_handle) in query.iter_mut() {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() && anim.running {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}
