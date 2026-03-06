use bevy::{
    audio::AudioSource,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    pbr::StandardMaterial,
    prelude::*,
    scene::Scene,
};

pub struct ProfilingPlugin;

impl Plugin for ProfilingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProfilingHudState::default())
            .add_systems(Startup, (log_profiling_mode, setup_profiling_hud))
            .add_systems(
                Update,
                (
                    toggle_profiling_hud,
                    update_profiling_hud,
                    sync_profiling_visibility,
                ),
            );
    }
}

#[derive(Resource)]
struct ProfilingHudState {
    visible: bool,
}

#[allow(clippy::derivable_impls)] // cfg!() evaluates at compile time; derive would always be false
impl Default for ProfilingHudState {
    fn default() -> Self {
        Self {
            visible: cfg!(feature = "profiling"),
        }
    }
}

#[derive(Component)]
struct ProfilingRoot;

#[derive(Component)]
struct ProfilingText;

fn log_profiling_mode() {
    if cfg!(feature = "profiling") {
        info!(
            "Profiling feature enabled. Tracy memory events are active; press F11 to toggle the profiling HUD."
        );
    }
}

fn setup_profiling_hud(mut commands: Commands, hud_state: Res<ProfilingHudState>) {
    let visibility = if hud_state.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    commands
        .spawn((
            ProfilingRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                top: Val::Px(84.0),
                bottom: Val::Auto,
                left: Val::Auto,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.72)),
            GlobalZIndex(i32::MAX),
            visibility,
        ))
        .with_child((
            ProfilingText,
            Text::new("Profiling HUD"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

fn toggle_profiling_hud(
    input: Res<ButtonInput<KeyCode>>,
    mut hud_state: ResMut<ProfilingHudState>,
) {
    if input.just_pressed(KeyCode::F11) {
        hud_state.visible = !hud_state.visible;
    }
}

fn sync_profiling_visibility(
    hud_state: Res<ProfilingHudState>,
    mut roots: Query<&mut Visibility, With<ProfilingRoot>>,
) {
    if !hud_state.is_changed() {
        return;
    }

    let Ok(mut visibility) = roots.single_mut() else {
        return;
    };

    *visibility = if hud_state.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn update_profiling_hud(
    diagnostics: Res<DiagnosticsStore>,
    entities: Query<Entity>,
    images: Res<Assets<Image>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    audio: Res<Assets<AudioSource>>,
    scenes: Res<Assets<Scene>>,
    mut text_query: Query<&mut Text, With<ProfilingText>>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let t = timer.get_or_insert_with(|| Timer::from_seconds(0.5, TimerMode::Repeating));
    t.tick(time.delta());
    if !t.just_finished() {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .map(|value| format!("{value:>4.0}"))
        .unwrap_or_else(|| " N/A".to_string());

    **text = format!(
        "FPS: {fps}\nEntities: {}\nImages: {}\nMeshes: {}\nMaterials: {}\nAudio: {}\nScenes: {}\nTracy memory: {}",
        entities.iter().len(),
        images.len(),
        meshes.len(),
        materials.len(),
        audio.len(),
        scenes.len(),
        if cfg!(feature = "profiling") {
            "enabled"
        } else {
            "run with --features profiling"
        },
    );
}
