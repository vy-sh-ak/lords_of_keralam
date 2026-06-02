use bevy::{prelude::*, text::DEFAULT_FONT_DATA, time::Time, window::RequestRedraw};

use super::MapConfigs;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct MapControlsPlugin;

#[derive(Component, PartialEq, Clone, Copy)]
enum MapSettingsButton {
    HeightInc,
    HeightDec,
    WidthInc,
    WidthDec,
    ScaleInc,
    ScaleDec,
    SeedInc,
    SeedDec,
    OffsetXInc,
    OffsetXDec,
    OffsetYInc,
    OffsetYDec,
    PersistenceInc,
    PersistenceDec,
    LacunarityInc,
    LacunarityDec,
    FrequencyInc,
    FrequencyDec,
}

#[derive(Component, PartialEq, Clone, Copy, Eq, Debug)]
enum MapSettingsType {
    Height,
    Width,
    Scale,
    Seed,
    OffsetX,
    OffsetY,
    Persistence,
    Lacunarity,
    Frequency,
}

impl MapSettingsType {
    fn label(&self) -> &str {
        match self {
            MapSettingsType::Height => "Height",
            MapSettingsType::Width => "Width",
            MapSettingsType::Scale => "Scale",
            MapSettingsType::Seed => "Seed",
            MapSettingsType::OffsetX => "Offset X",
            MapSettingsType::OffsetY => "Offset Y",
            MapSettingsType::Persistence => "Persistence",
            MapSettingsType::Lacunarity => "Lacunarity",
            MapSettingsType::Frequency => "Frequency",
        }
    }
}

#[derive(Resource, Default)]
struct HeldButton {
    button: Option<MapSettingsButton>,
    pressed_at: Option<f64>,
    last_repeat: Option<f64>,
}

impl Plugin for MapControlsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HeldButton::default())
            .add_systems(Startup, setup_controls)
            .add_systems(
                Update,
                (
                    button_system,
                    button_color_system,
                    button_repeat_system,
                    sync_setting_labels,
                ),
            );
    }
}

fn setup_controls(
    mut commands: Commands,
    map_configs: Res<MapConfigs>,
    mut fonts: ResMut<Assets<Font>>,
) {
    info!("Controls....");

    let font = fonts.add(Font::try_from_bytes(DEFAULT_FONT_DATA.to_vec()).unwrap());

    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                left: px(24),
                bottom: px(24),
                width: px(360),
                padding: UiRect::all(px(16)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12).with_alpha(0.85)),
            BorderColor::all(Color::WHITE.with_alpha(0.15)),
            ZIndex(10),
        ))
        .insert(children![
            build_settings_row(
                MapSettingsType::Height,
                MapSettingsButton::HeightDec,
                MapSettingsButton::HeightInc,
                setting_value(&map_configs, MapSettingsType::Height),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::Width,
                MapSettingsButton::WidthDec,
                MapSettingsButton::WidthInc,
                setting_value(&map_configs, MapSettingsType::Width),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::Scale,
                MapSettingsButton::ScaleDec,
                MapSettingsButton::ScaleInc,
                setting_value(&map_configs, MapSettingsType::Scale),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::Seed,
                MapSettingsButton::SeedDec,
                MapSettingsButton::SeedInc,
                setting_value(&map_configs, MapSettingsType::Seed),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::OffsetX,
                MapSettingsButton::OffsetXDec,
                MapSettingsButton::OffsetXInc,
                setting_value(&map_configs, MapSettingsType::OffsetX),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::OffsetY,
                MapSettingsButton::OffsetYDec,
                MapSettingsButton::OffsetYInc,
                setting_value(&map_configs, MapSettingsType::OffsetY),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::Persistence,
                MapSettingsButton::PersistenceDec,
                MapSettingsButton::PersistenceInc,
                setting_value(&map_configs, MapSettingsType::Persistence),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::Lacunarity,
                MapSettingsButton::LacunarityDec,
                MapSettingsButton::LacunarityInc,
                setting_value(&map_configs, MapSettingsType::Lacunarity),
                font.clone()
            ),
            build_settings_row(
                MapSettingsType::Frequency,
                MapSettingsButton::FrequencyDec,
                MapSettingsButton::FrequencyInc,
                setting_value(&map_configs, MapSettingsType::Frequency),
                font
            )
        ]);
}

fn build_settings_row(
    map_settings_type: MapSettingsType,
    dec: MapSettingsButton,
    inc: MapSettingsButton,
    value_text: String,
    font: Handle<Font>,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            height: px(32),
            ..default()
        },
        children![(
            Node {
                width: px(340),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            // Attach SettingType to the value label node, not the parent row
            children![
                (
                    Text::new(map_settings_type.label()),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                ),
                (
                    Button,
                    Node {
                        width: px(28),
                        height: px(28),
                        margin: UiRect::left(px(8)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                    dec,
                    children![(
                        Text::new("-"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                    )],
                ),
                (
                    Node {
                        width: px(96),
                        height: px(28),
                        margin: UiRect::horizontal(px(8)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    children![{
                        (
                            Text::new(value_text),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            map_settings_type,
                        )
                    }],
                ),
                (
                    Button,
                    Node {
                        width: px(28),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                    inc,
                    children![(
                        Text::new("+"),
                        TextFont {
                            font: font,
                            font_size: 18.0,
                            ..default()
                        },
                    )],
                )
            ],
        )],
    )
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &MapSettingsButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut map_configs: ResMut<MapConfigs>,
    mut held: ResMut<HeldButton>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs_f64();
    for (interaction, btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                trigger_button_action(btn, &mut map_configs);
                held.button = Some(*btn);
                held.pressed_at = Some(now);
                held.last_repeat = Some(now);
            }
            Interaction::None | Interaction::Hovered => {
                if held.button == Some(*btn) {
                    held.button = None;
                    held.pressed_at = None;
                    held.last_repeat = None;
                }
            }
        }
    }
}

fn trigger_button_action(btn: &MapSettingsButton, map_configs: &mut MapConfigs) {
    const FLOAT_STEP: f64 = 0.1;
    const OFFSET_STEP: f64 = 1.0;

    match btn {
        MapSettingsButton::HeightDec => {
            if map_configs.height > 1 {
                map_configs.height -= 1;
            }
        }
        MapSettingsButton::HeightInc => map_configs.height += 1,
        MapSettingsButton::WidthDec => {
            if map_configs.width > 1 {
                map_configs.width -= 1;
            }
        }
        MapSettingsButton::WidthInc => map_configs.width += 1,
        MapSettingsButton::ScaleDec => {
            if map_configs.scale > 1.0 {
                map_configs.scale -= 1.0;
            }
        }
        MapSettingsButton::ScaleInc => map_configs.scale += 1.0,
        MapSettingsButton::SeedDec => {
            map_configs.seed = map_configs.seed.saturating_sub(1);
        }
        MapSettingsButton::SeedInc => {
            map_configs.seed = map_configs.seed.saturating_add(1);
        }
        MapSettingsButton::OffsetXDec => map_configs.offset_x -= OFFSET_STEP,
        MapSettingsButton::OffsetXInc => map_configs.offset_x += OFFSET_STEP,
        MapSettingsButton::OffsetYDec => map_configs.offset_y -= OFFSET_STEP,
        MapSettingsButton::OffsetYInc => map_configs.offset_y += OFFSET_STEP,
        MapSettingsButton::PersistenceDec => {
            map_configs.persistence = (map_configs.persistence - FLOAT_STEP).max(0.0);
        }
        MapSettingsButton::PersistenceInc => map_configs.persistence += FLOAT_STEP,
        MapSettingsButton::LacunarityDec => {
            map_configs.lacunarity = (map_configs.lacunarity - FLOAT_STEP).max(0.0);
        }
        MapSettingsButton::LacunarityInc => map_configs.lacunarity += FLOAT_STEP,
        MapSettingsButton::FrequencyDec => {
            map_configs.frequency = (map_configs.frequency - FLOAT_STEP).max(0.1);
        }
        MapSettingsButton::FrequencyInc => map_configs.frequency += FLOAT_STEP,
    }
}

fn button_repeat_system(
    time: Res<Time>,
    mut held: ResMut<HeldButton>,
    mut map_configs: ResMut<MapConfigs>,
    mut request_redraw_writer: MessageWriter<RequestRedraw>,
) {
    if held.button.is_some() {
        request_redraw_writer.write(RequestRedraw);
    }
    const INITIAL_DELAY: f64 = 0.15;
    const REPEAT_RATE: f64 = 0.08;
    if let (Some(btn), Some(pressed_at)) = (held.button, held.pressed_at) {
        let now = time.elapsed_secs_f64();
        let since_pressed = now - pressed_at;
        let last_repeat = held.last_repeat.unwrap_or(pressed_at);
        let since_last = now - last_repeat;
        if since_pressed > INITIAL_DELAY && since_last > REPEAT_RATE {
            trigger_button_action(&btn, &mut map_configs);
            held.last_repeat = Some(now);
        }
    }
}

fn sync_setting_labels(
    map_configs: Res<MapConfigs>,
    mut query: Query<(&MapSettingsType, &mut Text)>,
) {
    if !map_configs.is_changed() {
        return;
    }

    for (setting_type, mut text) in &mut query {
        *text = Text::new(setting_value(&map_configs, *setting_type));
    }
}

fn setting_value(map_configs: &MapConfigs, setting_type: MapSettingsType) -> String {
    match setting_type {
        MapSettingsType::Height => map_configs.height.to_string(),
        MapSettingsType::Width => map_configs.width.to_string(),
        MapSettingsType::Scale => format!("{:.1}", map_configs.scale),
        MapSettingsType::Seed => map_configs.seed.to_string(),
        MapSettingsType::OffsetX => format!("{:.1}", map_configs.offset_x),
        MapSettingsType::OffsetY => format!("{:.1}", map_configs.offset_y),
        MapSettingsType::Persistence => format!("{:.1}", map_configs.persistence),
        MapSettingsType::Lacunarity => format!("{:.1}", map_configs.lacunarity),
        MapSettingsType::Frequency => format!("{:.1}", map_configs.frequency),
    }
}

fn button_color_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<MapSettingsButton>),
    >,
) {
    for (interaction, mut color) in &mut query {
        match *interaction {
            Interaction::Pressed => *color = PRESSED_BUTTON.into(),
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
