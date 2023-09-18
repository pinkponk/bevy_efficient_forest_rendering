use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

pub struct DebugFrameratePlugin;

impl Plugin for DebugFrameratePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LogDiagnosticsPlugin::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup_overlay)
            .add_systems(Update, update_fps);
    }
}

#[derive(Component)]
struct FpsText;

fn setup_overlay(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let ui_font = asset_server.load("fonts/Exo-Bold.otf");

    commands
        .spawn(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                bottom: Val::Px(25.0),
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: ui_font.clone(),
                            font_size: 30.0,
                            color: Color::GOLD,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("ui FPS"))
        .insert(FpsText);
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.0}", average);
                text.sections[1].style.color = match average {
                    x if x >= 50.0 => Color::GREEN,
                    x if x > 40.0 && x < 50.0 => Color::YELLOW,
                    x if x <= 40.0 => Color::RED,
                    _ => Color::WHITE,
                };
            }
        }
    }
}
