use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{AppState, AudioSettings, save::SaveState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(AppState::Menu), menu_setup)
            .add_systems(OnEnter(MenuState::Main), spawn_main_menu)
            .add_systems(OnEnter(MenuState::Settings), spawn_settings_menu)
            .add_systems(OnEnter(MenuState::LoadPlayer), spawn_load_player_menu)
            .add_systems(
                Update,
                (button_system, menu_action, sync_settings_labels).run_if(in_state(AppState::Menu)),
            );
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum MenuState {
    Main,
    Settings,
    LoadPlayer,
    #[default]
    Disabled,
}

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    LoadPlayer,
    SelectPlayer(String),
    Quit,
    ToggleMusic,
    ToggleSfx,
    BackToMain,
}

const BUTTON_NORMAL: Color = Color::srgba(0.15, 0.55, 0.15, 0.95);
const BUTTON_HOVERED: Color = Color::srgba(0.25, 0.70, 0.25, 1.0);
const BUTTON_PRESSED: Color = Color::srgba(0.10, 0.40, 0.10, 1.0);

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    font: Handle<Font>,
    label: &str,
    action: MenuButtonAction,
) {
    parent
        .spawn((
            action,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(48.0), Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(BUTTON_NORMAL),
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            DespawnOnExit(MenuState::Main),
            GlobalZIndex(100),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Rocket Lab"),
                TextFont {
                    font: font.clone(),
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            spawn_button(parent, font.clone(), "Play", MenuButtonAction::Play);
            #[cfg(not(target_arch = "wasm32"))]
            spawn_button(
                parent,
                font.clone(),
                "Load Player",
                MenuButtonAction::LoadPlayer,
            );
            spawn_button(parent, font.clone(), "Settings", MenuButtonAction::Settings);
            spawn_button(parent, font, "Quit", MenuButtonAction::Quit);
        });
}

fn spawn_settings_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let music_label = if audio_settings.music_enabled {
        "Music: ON"
    } else {
        "Music: OFF"
    };
    let sfx_label = if audio_settings.sfx_enabled {
        "SFX: ON"
    } else {
        "SFX: OFF"
    };

    commands
        .spawn((
            DespawnOnExit(MenuState::Settings),
            GlobalZIndex(100),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font: font.clone(),
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            spawn_button(
                parent,
                font.clone(),
                music_label,
                MenuButtonAction::ToggleMusic,
            );
            spawn_button(parent, font.clone(), sfx_label, MenuButtonAction::ToggleSfx);
            spawn_button(parent, font, "Back", MenuButtonAction::BackToMain);
        });
}

fn spawn_load_player_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    #[cfg(not(target_arch = "wasm32"))]
    let players = crate::save::list_players();
    #[cfg(target_arch = "wasm32")]
    let players: Vec<String> = Vec::new();

    commands
        .spawn((
            DespawnOnExit(MenuState::LoadPlayer),
            GlobalZIndex(100),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Load Player"),
                TextFont {
                    font: font.clone(),
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            if players.is_empty() {
                parent.spawn((
                    Text::new("No saved players yet"),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                ));
            } else {
                for name in &players {
                    spawn_button(
                        parent,
                        font.clone(),
                        name,
                        MenuButtonAction::SelectPlayer(name.clone()),
                    );
                }
            }

            spawn_button(parent, font, "Back", MenuButtonAction::BackToMain);
        });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(BUTTON_PRESSED);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVERED);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_NORMAL);
            }
        }
    }
}

fn sync_settings_labels(
    audio_settings: Res<AudioSettings>,
    button_query: Query<(&MenuButtonAction, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    if !audio_settings.is_changed() {
        return;
    }
    for (action, children) in &button_query {
        let label = match action {
            MenuButtonAction::ToggleMusic => {
                if audio_settings.music_enabled {
                    "Music: ON"
                } else {
                    "Music: OFF"
                }
            }
            MenuButtonAction::ToggleSfx => {
                if audio_settings.sfx_enabled {
                    "SFX: ON"
                } else {
                    "SFX: OFF"
                }
            }
            _ => continue,
        };
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = label.to_string();
            }
        }
    }
}

#[derive(SystemParam)]
struct MenuLoadParams<'w> {
    save_state: ResMut<'w, SaveState>,
    collection: ResMut<'w, crate::save::RocketCollection>,
    player_balance: ResMut<'w, crate::save::PlayerBalance>,
    owned_materials: ResMut<'w, crate::save::OwnedMaterials>,
    rocket_cam_owned: ResMut<'w, crate::save::RocketCamOwned>,
    creative_mode_owned: ResMut<'w, crate::save::CreativeModeOwned>,
    rocket_dims: ResMut<'w, crate::rocket::RocketDimensions>,
    flight_params: ResMut<'w, crate::rocket::RocketFlightParameters>,
    equipped: ResMut<'w, crate::inventory::EquippedLoadout>,
    inventory: ResMut<'w, crate::inventory::Inventory>,
    owned_motor_sizes: ResMut<'w, crate::inventory::OwnedMotorSizes>,
    owned_tube_types: ResMut<'w, crate::inventory::OwnedTubeTypes>,
    owned_nosecone_types: ResMut<'w, crate::inventory::OwnedNoseconeTypes>,
    experience: ResMut<'w, crate::inventory::PlayerExperience>,
    launch_history: ResMut<'w, crate::save::LaunchHistory>,
}

#[allow(unused_variables)]
fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut audio_settings: ResMut<AudioSettings>,
    mut load_params: MenuLoadParams,
    mut app_exit: MessageWriter<AppExit>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MenuButtonAction::Play => {
                app_state.set(AppState::Lab);
                menu_state.set(MenuState::Disabled);
            }
            MenuButtonAction::Settings => {
                menu_state.set(MenuState::Settings);
            }
            MenuButtonAction::LoadPlayer => {
                menu_state.set(MenuState::LoadPlayer);
            }
            #[cfg(not(target_arch = "wasm32"))]
            MenuButtonAction::SelectPlayer(name) => {
                load_params.save_state.player_name = name.clone();
                if let Ok(meta) = crate::save::load_player_meta(name) {
                    load_params.player_balance.0 = meta.balance;
                    load_params.owned_materials.0 = meta.owned_materials;
                    load_params.rocket_cam_owned.0 = meta.rocket_cam_owned;
                    *load_params.inventory = meta.inventory;
                    load_params.owned_motor_sizes.0 = meta.owned_motor_sizes;
                    load_params.owned_tube_types.0 = meta.owned_tube_types;
                    load_params.owned_nosecone_types.0 = meta.owned_nosecone_types;
                    load_params.experience.0 = meta.experience;
                    *load_params.launch_history = meta.launch_history;
                    load_params.creative_mode_owned.0 = meta.creative_mode_owned;
                }
                load_params.collection.rockets.clear();
                load_params.collection.active = None;
                let rocket_names = crate::save::list_rockets(name);
                for rname in &rocket_names {
                    if let Ok(data) = crate::save::load_rocket(name, rname) {
                        load_params.collection.rockets.push(data);
                    }
                }
                if !load_params.collection.rockets.is_empty() {
                    load_params.collection.active = Some(0);
                    let rocket = &load_params.collection.rockets[0];
                    *load_params.rocket_dims = rocket.dimensions.clone();
                    load_params.rocket_dims.flag_changed = true;
                    *load_params.flight_params = rocket.flight_params.clone();
                    *load_params.equipped = rocket.equipped.clone();
                    load_params.save_state.rocket_name_buf = rocket.name.clone();
                }
                app_state.set(AppState::Lab);
                menu_state.set(MenuState::Disabled);
            }
            #[cfg(target_arch = "wasm32")]
            MenuButtonAction::SelectPlayer(_) => {}
            MenuButtonAction::Quit => {
                app_exit.write(AppExit::Success);
            }
            MenuButtonAction::ToggleMusic => {
                audio_settings.music_enabled = !audio_settings.music_enabled;
            }
            MenuButtonAction::ToggleSfx => {
                audio_settings.sfx_enabled = !audio_settings.sfx_enabled;
            }
            MenuButtonAction::BackToMain => {
                menu_state.set(MenuState::Main);
            }
        }
    }
}
