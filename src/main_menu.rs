use bevy::prelude::*;

use crate::{state::GameState, ui::UiButton};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(build_play_button))
            .add_system_set(
                SystemSet::on_update(GameState::MainMenu).with_system(play_button_interaction),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::MainMenu).with_system(destroy_play_button),
            )
            .add_system_set(SystemSet::on_enter(GameState::Pause).with_system(build_play_button))
            .add_system_set(
                SystemSet::on_update(GameState::Pause).with_system(play_button_interaction),
            )
            .add_system_set(SystemSet::on_exit(GameState::Pause).with_system(destroy_play_button));
    }
}

#[derive(Component)]
pub struct PlayButton;

fn play_button_interaction(
    mut state: ResMut<State<GameState>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
) {
    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => match state.current() {
                GameState::MainMenu => {
                    state.set(GameState::Play).unwrap();
                }
                GameState::Pause => {
                    state.pop().unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn build_play_button(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            image: UiImage(asset_server.load("play_button.png")),
            ..default()
        })
        .insert(UiButton)
        .insert(PlayButton);
}

fn destroy_play_button(mut commands: Commands, ui_components: Query<Entity, With<PlayButton>>) {
    for entity in ui_components.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
