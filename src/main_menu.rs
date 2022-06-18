use bevy::prelude::*;

use crate::state::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(build_main_menu))
            .add_system_set(
                SystemSet::on_update(GameState::MainMenu).with_system(interaction_system),
            )
            .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(destroy_main_menu));
    }
}

#[derive(Component)]
struct PlayButton;

fn interaction_system(
    mut state: ResMut<State<GameState>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => state.set(GameState::Play).unwrap(),
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn build_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            image: UiImage(asset_server.load("h")),
            ..default()
        })
        .insert(PlayButton);
}

fn destroy_main_menu(mut commands: Commands, ui_components: Query<Entity, With<PlayButton>>) {
    for entity in ui_components.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
