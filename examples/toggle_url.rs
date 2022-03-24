use bevy::prelude::*;
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WebviewPlugin::new().register_engine(webview_engine::headless))
        .add_startup_system(setup)
        .add_system(toggle_system)
        .run();
}

const URLS: [&'static str; 3] = [
    "webview:///intro.html",
    "webview:///about.html",
    "https://bevyengine.org/",
];

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    // webview
    commands.spawn_bundle(WebviewUIBundle {
        webview: Webview {
            uri: Some(String::from(URLS[0])),
            ..Default::default()
        },
        style: Style {
            size: Size::new(Val::Percent(80.0), Val::Percent(80.)),
            margin: Rect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    });

    // reload button
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Next",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });

    commands.insert_resource(CurrentUrl(0));
}

struct CurrentUrl(usize);

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn toggle_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut webview_commands: WebviewEventWriter<WebviewCommand>,
    keyboard_input: Res<Input<KeyCode>>,
    mut current_url: ResMut<CurrentUrl>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                current_url.0 += 1;

                if current_url.0 >= URLS.len() {
                    current_url.0 = 0;
                }

                webview_commands.send(WebviewCommand::LoadUri(URLS[current_url.0].to_string()));
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
