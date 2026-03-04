use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

#[derive(Component)]
pub struct FpsRoot;

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct FpsValue;

pub fn setup_fps_counter(mut commands: Commands) {
    let root = commands
        .spawn((
            FpsRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.),
                top: Val::Px(8.),
                bottom: Val::Auto,
                left: Val::Auto,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..Default::default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.4)),
            GlobalZIndex(i32::MAX),
        ))
        .id();

    let text_fps = commands
        .spawn((
            FpsText,
            Text::new("FPS: "),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .with_child((
            FpsValue,
            TextSpan::new(" N/A"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();
    commands.entity(root).add_children(&[text_fps]);
}

pub fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<(&mut TextSpan, &mut TextColor), With<FpsValue>>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let t = timer.get_or_insert_with(|| Timer::from_seconds(0.25, TimerMode::Repeating));
    t.tick(time.delta());
    if !t.just_finished() {
        return;
    }

    for (mut text, mut color) in &mut query {
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            **text = format!("{value:>4.0}");

            color.0 = if value >= 120.0 {
                Color::srgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                Color::srgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
            } else if value >= 30.0 {
                Color::srgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
            } else {
                Color::srgb(1.0, 0.0, 0.0)
            }
        } else {
            **text = " N/A".into();
            color.0 = Color::WHITE;
        }
    }
}

pub fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12)
        && let Ok(mut vis) = q.single_mut()
    {
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}
