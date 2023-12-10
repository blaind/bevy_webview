use bevy::prelude::*;
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WebviewPlugin::new().register_engine(webview_engine::headless))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_system)
        .add_systems(PostUpdate, hide_btn_visibility)
        .run();
}

#[derive(Component)]
struct VisibilityToggleButton;

#[derive(Component)]
struct SpawnToggleButton;

#[derive(Component)]
struct WebviewRoot;

fn get_webview() -> WebviewUIBundle {
    WebviewUIBundle {
        webview: Webview {
            uri: Some(String::from("https://bevyengine.org/")),
            ..Default::default()
        },
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(60.0),
                height: Val::Percent(80.0),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(get_webview());
        })
        .insert(WebviewRoot);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(10.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // toggle visibility
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Hide",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 24.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(VisibilityToggleButton);

            // destroy
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Despawn",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 24.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(SpawnToggleButton);
        });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn toggle_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &Children,
            Option<&SpawnToggleButton>,
            Option<&VisibilityToggleButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut webview_commands: WebviewEventWriter<WebviewCommand>,
    keyboard_input: Res<Input<KeyCode>>,
    mut text_query: Query<&mut Text>,
    mut webview: Query<(Entity, &mut Visibility), With<Webview>>,
    mut commands: Commands,
    webview_root: Query<Entity, With<WebviewRoot>>,
) {
    for (interaction, mut color, children, spawn_toggle, visibility_toggle) in
        interaction_query.iter_mut()
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        if spawn_toggle.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    if webview.is_empty() {
                        commands
                            .entity(webview_root.single())
                            .with_children(|parent| {
                                parent.spawn(get_webview());
                            });

                        text.sections[0].value = "Despawn".to_string();
                    } else {
                        let (entity, _) = webview.single();
                        commands.entity(entity).despawn_recursive();
                        text.sections[0].value = "Spawn".to_string();
                    }
                    *color = PRESSED_BUTTON.into()
                }
                Interaction::Hovered => *color = HOVERED_BUTTON.into(),
                Interaction::None => *color = NORMAL_BUTTON.into(),
            }
        }

        if visibility_toggle.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    if webview.is_empty() {
                    } else {
                        let (_, mut visibility) = webview.single_mut();
                        if *visibility == Visibility::Inherited {
                            text.sections[0].value = "Show".to_string();
                            *visibility = Visibility::Hidden;
                        } else {
                            text.sections[0].value = "Hide".to_string();
                            *visibility = Visibility::Inherited;
                        }
                    }
                    *color = PRESSED_BUTTON.into()
                }
                Interaction::Hovered => *color = HOVERED_BUTTON.into(),
                Interaction::None => *color = NORMAL_BUTTON.into(),
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::F5) {
        webview_commands.send(WebviewCommand::Reload);
    }
}

fn hide_btn_visibility(
    webview: Query<Entity, With<Webview>>,
    button: Query<(Entity, &Children), With<VisibilityToggleButton>>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let (button_entity, _) = button.single();
    let mut visibility = visibility_query.get_mut(button_entity).unwrap();
    let should_be_visible = !webview.is_empty();
    if (*visibility == Visibility::Inherited) != should_be_visible {
        if should_be_visible {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
