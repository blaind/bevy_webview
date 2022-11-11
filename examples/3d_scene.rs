use bevy::{log, prelude::*};
use bevy_webview::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WebviewPlugin::with_engine(webview_engine::headless))
        .add_startup_system(setup)
        .add_system(rotator)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0)
            .looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        ..Default::default()
    });
    // webview
    commands.spawn_bundle(WebviewBundle {
        webview: Webview {
            uri: Some(String::from("https://html5test.com/")),
            color: Color::rgba(0.3, 0.3, 0.3, 0.5),
            ..Default::default()
        },
        size: WebviewSize {
            x: 3.,
            y: 2.,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.5, 0.0),
            rotation: Quat::from_rotation_y(-std::f32::consts::PI / 3.0),
            ..Default::default()
        },
        ..Default::default()
    });

    log::error!("Note that 3D canvas interaction is not implemented yet, please provide feedback of needs at https://github.com/blaind/bevy_webview/issues/1");
}

fn rotator(time: Res<Time>, mut query: Query<&mut Transform, With<Webview>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_y(0.05 * time.delta_seconds());
    }
}
