use bevy::prelude::*;
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WebviewPlugin::new().register_engine(webview_engine::headless))
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(WebviewUIBundle {
        webview: Webview {
            uri: Some(String::from("http://bevyengine.org/")),
            color: Color::rgba_u8(35, 35, 38, 255),
            ..Default::default()
        },
        style: Style {
            size: Size::new(Val::Percent(30.0), Val::Percent(100.)),
            ..Default::default()
        },
        ..Default::default()
    });
}
