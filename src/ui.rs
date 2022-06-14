use bevy::prelude::*;

use crate::player::{Latency, Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_ui)
            .add_system(update_latency_text);
    }
}

#[derive(Component)]
struct LatencyText;

fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("VT323-Regular.ttf");
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            text: Text::with_section(
                "0ms",
                TextStyle {
                    font,
                    font_size: 50.0,
                    color: Color::BLACK,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..default()
                },
            ),
            ..default()
        })
        .insert(LatencyText);
}

fn update_latency_text(
    latency: Res<Latency>,
    mut text_query: Query<&mut Text, With<LatencyText>>,
    mut player_query: Query<&mut Player>,
) {
    let mut text = text_query.single_mut();
    let player = match player_query.get_single_mut() {
        Ok(p) => p,
        Err(e) => {
            text.sections[0].value = "NaNms".to_string();
            return;
        }
    };

    text.sections[0].value = ((latency.0.iter().sum::<i32>() as f32 / latency.0.len() as f32)
        * player.latency as f32)
        .to_string()
        + "ms"
}
