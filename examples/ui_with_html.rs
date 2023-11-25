use bevy::prelude::*;
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WebviewPlugin::new().register_engine(webview_engine::headless))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(WebviewUIBundle {
        webview: Webview {
            html: Some(include_str!("ui_with_html.html").to_string()),
            color: Color::rgb_u8(255, 228, 196),
            ..Default::default()
        },
        style: Style {
            width: Val::Percent(30.),
            height: Val::Percent(100.),
            ..Default::default()
        },
        ..Default::default()
    });
}
