use bevy::{
    log,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use headless_webview::types::WindowSize;

use crate::{
    types::{EventTransport, LaunchEvent, WebviewAction},
    Webview, WebviewSize,
};

/// This system takes care of initialing required `PbrBundle` for the webview
pub(crate) fn create_webview_system(
    event_transport: ResMut<EventTransport>,
    added_webviews: Query<
        (
            Entity,
            &Webview,
            &Transform,
            &GlobalTransform,
            Option<&Node>,
            Option<&WebviewSize>,
        ),
        Added<Webview>,
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, webview, transform, global_transform, node, webview_size) in added_webviews.iter()
    {
        if let Some(node) = node {
            let window_size =
                WindowSize::new(node.size.x.round() as u32, node.size.y.round() as u32);

            log::debug!(
                "Webview {:?} (UI) added, window_size={:?}, texture_size_mb={:.2}",
                entity,
                node.size,
                (window_size.width as usize * window_size.height as usize * 4) as f32
                    / 1024.
                    / 1024.
            );

            event_transport
                .webview_action_tx
                .send(WebviewAction::Launch(LaunchEvent {
                    entity,
                    webview: webview.clone(),
                    size: window_size.clone(),
                }))
                .unwrap();

            // UI node - insert placeholder image
            commands
                .entity(entity)
                .insert(UiImage(
                    images.add(Image::new(
                        Extent3d {
                            width: window_size.width as u32,
                            height: window_size.height as u32,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        [255, 255, 255, 255]
                            .repeat(window_size.width as usize * window_size.height as usize),
                        TextureFormat::Rgba8Unorm,
                    )),
                ))
                .insert(UiColor(webview.color));
        } else if let Some(webview_size) = webview_size {
            log::debug!(
                "Webview {:?} (PBR) added, texture_size_mb={:.2}",
                entity,
                webview_size.texture_byte_size_rgba_mb()
            );

            event_transport
                .webview_action_tx
                .send(WebviewAction::Launch(LaunchEvent {
                    entity,
                    webview: webview.clone(),
                    size: WindowSize::new(webview_size.pixels_x(), webview_size.pixels_y()),
                }))
                .unwrap();

            // PBR node
            let material_handle = materials.add(StandardMaterial {
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                base_color: webview.color,
                ..Default::default()
            });

            let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                webview_size.x,
                webview_size.y,
            ))));

            commands.entity(entity).insert_bundle(PbrBundle {
                mesh: quad_handle.clone(),
                material: material_handle,
                transform: *transform,
                global_transform: *global_transform,
                ..Default::default()
            });
        } else {
            log::warn!(
                "Webview {:?} added, but no `Node` or `WebviewSize` component found - not launched",
                entity
            );
        }
    }
}
