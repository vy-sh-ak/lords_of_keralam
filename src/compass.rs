use bevy::{prelude::*, ui::{widget::ImageNode, UiTransform}};

use crate::camera_config::WorldDirection;

#[derive(Component)]
pub struct CompassMarker;

#[derive(Component)]
pub struct CompassNeedle;

pub fn spawn_compass(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Px(50.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                top: Val::Px(12.0),
                ..default()
            },
            CompassMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode::new(asset_server.load("compass.png")),
                CompassNeedle,
            ));
        });
}

pub fn update_compass_system(
    world_direction: Res<WorldDirection>,
    mut compass_transform: Single<&mut UiTransform, With<CompassNeedle>>,
) {
    compass_transform.rotation = world_direction.compass_rotation();
}