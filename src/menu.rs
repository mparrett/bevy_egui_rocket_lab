use bevy::prelude::*;

use crate::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(Update, menu_button_system.run_if(in_state(AppState::Menu)));
    }
}

#[derive(Component)]
struct LaunchButton;

const BUTTON_NORMAL: Color = Color::srgba(0.15, 0.55, 0.15, 0.95);
const BUTTON_HOVERED: Color = Color::srgba(0.25, 0.70, 0.25, 1.0);
const BUTTON_PRESSED: Color = Color::srgba(0.10, 0.40, 0.10, 1.0);

fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            DespawnOnExit(AppState::Menu),
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

            parent
                .spawn((
                    LaunchButton,
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
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Launch"),
                        TextFont {
                            font,
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<LaunchButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut bg) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(BUTTON_PRESSED);
                next_state.set(AppState::Playing);
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
