use std::time::Duration;

use bevy::prelude::*;
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WebviewPlugin::with_engine(webview_engine::headless))
        .add_systems(Startup, setup)
        .add_systems(Update, send_commands_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(WebviewUIBundle {
        webview: Webview {
            uri: Some("https://bevyengine.org/".into()),
            ..Default::default()
        },
        style: Style {
            width: Val::Percent(50.),
            height: Val::Percent(50.),
            // margin: Rect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        visibility: Visibility::Inherited,
        ..Default::default()
    });

    commands.insert_resource(Elapsed {
        iteration: 0,
        timer: Timer::new(Duration::from_millis(2000), TimerMode::Repeating),
    });
}

#[derive(Component, Resource)]
struct Elapsed {
    iteration: usize,
    timer: Timer,
}

fn send_commands_system(
    mut webview_commands: WebviewEventWriter<WebviewCommand>,
    time: Res<Time>,
    mut elapsed: ResMut<Elapsed>,
) {
    // events to send every elapsed.timer ticks
    let events = [
        WebviewCommand::Reload,
        WebviewCommand::LoadHtml("<body style=\"background-color: yellow\">LoadHtml</body>".into()),
        WebviewCommand::RunJavascript("document.body.innerHTML = 'Hello Javascript!';".into()),
        WebviewCommand::LoadHtml(
            "<body style=\"background-color: blue\">Blue background</body>".into(),
        ),
        WebviewCommand::LoadUri("https://www.google.com/".into()),
    ];

    if elapsed.timer.tick(time.delta()).just_finished() {
        if let Some(event) = events.get(elapsed.iteration) {
            // this command is sent to all webviews. There's also a method for sending to a specific entity-webview
            webview_commands.send(event.clone());
        }

        elapsed.iteration += 1;
    }
}
