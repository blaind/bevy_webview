use bevy::prelude::*;
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WebviewPlugin::new().register_engine(webview_engine::headless))
        .add_systems(Startup, setup)
        .add_systems(Update, reload_system)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // webview
    commands.spawn(WebviewUIBundle {
        webview: Webview {
            uri: Some(String::from("webview:///test_webview.html")),
            ..Default::default()
        },
        style: Style {
            width: Val::Percent(80.0),
            height: Val::Percent(80.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        visibility: Visibility::Inherited,
        ..Default::default()
    });

    // reload button
    commands
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                right: Val::Percent(10.),
                bottom: Val::Percent(10.),
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Reload",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
        });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn reload_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut webview_commands: WebviewEventWriter<WebviewCommand>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                webview_commands.send(WebviewCommand::Reload);
                *color = PRESSED_BUTTON.into()
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }

    if keyboard_input.just_pressed(KeyCode::F5) {
        webview_commands.send(WebviewCommand::Reload);
    }
}
