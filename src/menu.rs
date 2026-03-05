use bevy::prelude::*;

use crate::{AppState, AudioSettings};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(AppState::Menu), menu_setup)
            .add_systems(OnEnter(MenuState::Main), spawn_main_menu)
            .add_systems(OnEnter(MenuState::Settings), spawn_settings_menu)
            .add_systems(
                Update,
                (button_system, menu_action, sync_settings_labels)
                    .run_if(in_state(AppState::Menu)),
            );
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum MenuState {
    Main,
    Settings,
    #[default]
    Disabled,
}

#[derive(Component)]
enum MenuButtonAction {
    Launch,
    Settings,
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

fn spawn_button(parent: &mut ChildSpawnerCommands, font: Handle<Font>, label: &str, action: MenuButtonAction) {
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

            spawn_button(parent, font.clone(), "Play", MenuButtonAction::Launch);
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
    let music_label = if audio_settings.music_enabled { "Music: ON" } else { "Music: OFF" };
    let sfx_label = if audio_settings.sfx_enabled { "SFX: ON" } else { "SFX: OFF" };

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

            spawn_button(parent, font.clone(), music_label, MenuButtonAction::ToggleMusic);
            spawn_button(parent, font.clone(), sfx_label, MenuButtonAction::ToggleSfx);
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
                if audio_settings.music_enabled { "Music: ON" } else { "Music: OFF" }
            }
            MenuButtonAction::ToggleSfx => {
                if audio_settings.sfx_enabled { "SFX: ON" } else { "SFX: OFF" }
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

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut audio_settings: ResMut<AudioSettings>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MenuButtonAction::Launch => {
                app_state.set(AppState::Lab);
                menu_state.set(MenuState::Disabled);
            }
            MenuButtonAction::Settings => {
                menu_state.set(MenuState::Settings);
            }
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
