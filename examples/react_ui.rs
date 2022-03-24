use std::net::TcpStream;

use bevy::{log, prelude::*};
use bevy_webview::prelude::*;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WebviewPlugin::with_engine(webview_engine::headless))
        .add_webview_input_event::<CubeSpawn>("spawn_cubes")
        .add_webview_input_event::<CubeDestroy>("destroy_cubes")
        .add_webview_output_event::<CubeCount>("cube_count")
        .add_startup_system(setup_scene)
        .add_startup_system(setup_webviews)
        .add_system(cube_spawn_system)
        .add_system(cube_destroy_system)
        .run();
}

#[derive(Serialize)]
pub struct CubeCount(usize);

fn setup_webviews(mut commands: Commands) {
    const REACT_SERVER_ADDR: &str = "localhost:3000";
    if let Err(e) = TcpStream::connect(REACT_SERVER_ADDR) {
        log::error!(
            "Make sure that you first launch React in dev mode in folder `examples/react-ui` (host={:?}, reason={:?})",
            REACT_SERVER_ADDR,
            e.to_string()
        );

        std::process::exit(255);
    }

    // 3d webview
    commands.spawn_bundle(WebviewBundle {
        webview: Webview {
            uri: Some(String::from("https://bevyengine.org")),
            color: Color::rgba(0., 0., 0., 0.3),
            ..Default::default()
        },
        size: WebviewSize {
            x: 3.,
            y: 2.,
            ppu: 300.,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.5, 2.5, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // UI webview (REACT)
    commands.spawn_bundle(WebviewUIBundle {
        webview: Webview {
            uri: Some(String::from("http://localhost:3000/")),
            color: Color::rgba(0., 0., 0., 0.0),
            ..Default::default()
        },
        style: Style {
            size: Size::new(Val::Percent(30.0), Val::Percent(80.)),
            margin: Rect::all(Val::Px(50.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0)
            .looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
}

#[derive(Deserialize, Debug)]
pub struct CubeSpawn {
    count: usize,
}

#[derive(Component)]
pub struct Cube;

fn cube_spawn_system(
    mut cube_spawn_events: WebviewEventReader<CubeSpawn>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cubes: Query<Entity, With<Cube>>,
    mut cube_count_events: WebviewEventWriter<CubeCount>,
) {
    let mut added_count = 0;
    for event in cube_spawn_events.iter() {
        let mut rng = thread_rng();

        added_count += event.count;

        for _ in 0..event.count {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.3 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(
                        (rng.gen::<f32>() - 0.5) * 2.0,
                        (rng.gen::<f32>()) * 2.0 + 1.0,
                        (rng.gen::<f32>()) * 2.0,
                    ),
                    ..Default::default()
                })
                .insert(Cube);
        }
    }

    if added_count > 0 {
        cube_count_events.send(CubeCount(cubes.iter().count() + added_count));
    }
}

#[derive(Deserialize, Debug)]
pub struct CubeDestroy;

fn cube_destroy_system(
    mut cube_destroy_events: WebviewEventReader<CubeDestroy>,
    mut commands: Commands,
    cubes: Query<Entity, With<Cube>>,
    mut cube_count_events: WebviewEventWriter<CubeCount>,
) {
    if cube_destroy_events.iter().next().is_some() {
        cubes
            .iter()
            .for_each(|cube| commands.entity(cube).despawn());

        cube_count_events.send(CubeCount(0));
    }
}
