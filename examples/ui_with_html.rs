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
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(WebviewUIBundle {
        webview: Webview {
            html: Some(include_str!("ui_with_html.html").to_string()),
            color: Color::rgb_u8(255, 228, 196),
            ..Default::default()
        },
        style: Style {
            size: Size::new(Val::Percent(30.0), Val::Percent(100.)),
            ..Default::default()
        },
        ..Default::default()
    });
}
