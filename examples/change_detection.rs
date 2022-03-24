use std::time::Duration;

use bevy::{log, prelude::*};
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WebviewPlugin::with_engine(webview_engine::headless))
        .add_startup_system(setup)
        .add_system(change_webview_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(Elapsed {
        iteration: 0,
        timer: Timer::new(Duration::from_millis(2000), true),
    });
}

#[derive(Component, Debug)]
struct Elapsed {
    iteration: usize,
    timer: Timer,
}

fn change_webview_system(
    mut webviews: Query<(Entity, &mut Webview)>,
    time: Res<Time>,
    mut elapsed: ResMut<Elapsed>,
    mut commands: Commands,
) {
    if elapsed.timer.tick(time.delta()).just_finished() {
        if elapsed.iteration == 0 {
            // at first tick, spawn the webview
            commands.spawn_bundle(WebviewUIBundle {
                webview: Webview {
                    uri: Some("https://bevyengine.org/".into()),
                    ..Default::default()
                },
                style: Style {
                    size: Size::new(Val::Percent(50.0), Val::Percent(50.)),
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            });

            elapsed.iteration += 1;
            return;
        } else if elapsed.iteration == 5 {
            // final iteration, exit
            log::info!("Done - exit(0)!");
            std::process::exit(0);
        }

        let (entity, mut webview) = match webviews.get_single_mut() {
            Ok(w) => w,
            Err(_) => return,
        };

        if elapsed.iteration == 1 {
            // change from URI to HTML mode
            webview.uri = None;
            webview.html =
                Some("<body style=\"background-color: yellow\">First iteration</body>".into());
        } else if elapsed.iteration == 2 {
            // change html content
            webview.html =
                Some("<body style=\"background-color: blue\">Second iteration</body>".into());
        } else if elapsed.iteration == 3 {
            webview.uri = Some("https://www.google.com/".into());
        } else if elapsed.iteration == 4 {
            // destroy the webview
            commands.entity(entity).despawn_recursive();
        }

        elapsed.iteration += 1;
    }
}
