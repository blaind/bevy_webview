use std::collections::HashMap;

use bevy::log;
use bevy::render::render_resource::{TextureDimension, TextureFormat};
use bevy::{prelude::*, render::render_resource::Extent3d};
use headless_webview::types::Texture;

use crate::types::{EventTransport, TextureReceivedEvent};
use crate::{Webview, WebviewState};

/// This system receives the webview textures, and updates bevy texture accordingly
///
/// If texture is not created or has been resized, old one is disposed and a new one will be created
pub(crate) fn update_webview_textures(
    event_transport: ResMut<EventTransport>,
    mut webviews: Query<
        (
            Entity,
            Option<&Handle<StandardMaterial>>,
            Option<&Node>,
            Option<&mut UiImage>,
            Option<&mut BackgroundColor>,
            &mut WebviewState,
        ),
        With<Webview>,
    >,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    // get only latest per entity
    let mut texture_events: HashMap<Entity, TextureReceivedEvent> = HashMap::new();
    for texture_event in event_transport.texture_rx.try_iter() {
        texture_events.insert(texture_event.entity, texture_event);
    }

    for texture_event in texture_events.values() {
        let (entity, material_handle, node, ui_image, ui_color, mut webview_state) =
            match webviews.get_mut(texture_event.entity) {
                Ok(v) => v,
                Err(_) => continue,
            };

        match (material_handle, node, ui_image) {
            // PbrBundle
            (Some(material_handle), None, None) => {
                let material = match standard_materials.get_mut(material_handle) {
                    Some(v) => v,
                    None => continue,
                };

                let image: Option<&mut Image> = material
                    .base_color_texture
                    .as_ref()
                    .and_then(|v| images.get_mut(v));

                let mut needs_new_texture = true;

                if let Some(image) = image {
                    if webview_state.texture_added
                        && try_apply_webview_texture_to_image(&texture_event, image)
                    {
                        needs_new_texture = false;
                    }
                }

                log::debug!(
                    "Webview {:?} (PBR) texture received (w={}, h={}), needs_new_texture={}",
                    entity,
                    texture_event.texture.width,
                    texture_event.texture.height,
                    needs_new_texture
                );

                /////// ABSTRACT //////
                if needs_new_texture || !webview_state.texture_added {
                    let mut material = standard_materials.get_mut(material_handle).unwrap();

                    let mut new_image = Image::new(
                        Extent3d {
                            width: texture_event.texture.width as u32,
                            height: texture_event.texture.height as u32,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        vec![
                            0u8;
                            texture_event.texture.width as usize
                                * texture_event.texture.height as usize
                                * 4
                        ],
                        TextureFormat::Rgba8UnormSrgb,
                    );

                    apply_buffer_to_texture_buffer(&texture_event.texture, &mut new_image);

                    if let Some(base_color_texture) = material.base_color_texture.as_ref() {
                        images.remove(base_color_texture);
                    }

                    let image_handle = images.add(new_image);
                    /////// ABSTRACT //////

                    material.base_color_texture = Some(image_handle);

                    if !webview_state.texture_added {
                        material.base_color = Color::default();
                        webview_state.texture_added = true;
                    }
                }
            }

            // UI bundle
            (None, Some(_ui_node), mut ui_image) => {
                let image = match &ui_image {
                    Some(ui_image) => images.get_mut(&ui_image.0),
                    None => None,
                };

                let mut needs_new_texture = true;

                if let Some(image) = image {
                    if webview_state.texture_added
                        && try_apply_webview_texture_to_image(&texture_event, image)
                    {
                        needs_new_texture = false;
                    } else {
                        images.remove(ui_image.as_ref().unwrap().0.clone()); // unwrap ok, checked before
                        ui_image = None;
                    }
                }

                log::debug!(
                    "Webview {:?} (UI) texture received (w={}, h={}), needs_new_texture={}",
                    entity,
                    texture_event.texture.width,
                    texture_event.texture.height,
                    needs_new_texture
                );

                /////// ABSTRACT //////
                if needs_new_texture {
                    let mut new_image = Image::new(
                        Extent3d {
                            width: texture_event.texture.width as u32,
                            height: texture_event.texture.height as u32,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        vec![
                            0u8;
                            texture_event.texture.width as usize
                                * texture_event.texture.height as usize
                                * 4
                        ],
                        TextureFormat::Rgba8UnormSrgb,
                    );

                    apply_buffer_to_texture_buffer(&texture_event.texture, &mut new_image);

                    let image_handle = images.add(new_image);

                    /////// ABSTRACT //////
                    match &mut ui_image {
                        Some(ui_image) => ui_image.0 = image_handle,
                        None => {
                            commands
                                .entity(texture_event.entity)
                                .insert(UiImage(image_handle));
                        }
                    }

                    if !webview_state.texture_added {
                        ui_color.unwrap().0 = Color::default();
                        webview_state.texture_added = true;
                    }
                }
            }

            _ => {
                log::warn!("Unknown webview texture combination");
            }
        };
    }
}

fn try_apply_webview_texture_to_image(
    webview_texture_event: &TextureReceivedEvent,
    image: &mut Image,
) -> bool {
    if webview_texture_event.texture.width as u32 != image.texture_descriptor.size.width
        || webview_texture_event.texture.height as u32 != image.texture_descriptor.size.height
    {
        return false;
    }

    apply_buffer_to_texture_buffer(&webview_texture_event.texture, image)
}

fn apply_buffer_to_texture_buffer(texture: &Texture, image: &mut Image) -> bool {
    match (&texture.format, &image.texture_descriptor.format) {
        (headless_webview::types::TextureFormat::Rgb8, TextureFormat::Rgba8UnormSrgb) => {
            // TODO can this be a performance bottleneck - par-iter?
            for (val, other) in image.data.chunks_mut(4).zip(texture.data.chunks(3)) {
                val[0] = other[0];
                val[1] = other[1];
                val[2] = other[2];
                val[3] = 255;
            }

            true
        }

        (headless_webview::types::TextureFormat::Rgba8, TextureFormat::Rgba8UnormSrgb) => {
            if image.data.len() == texture.data.len() {
                image.data.copy_from_slice(&texture.data);
            } else {
                log::error!("Webview update failed, mismatched texture dimensions!");
            }

            true
        }

        _ => {
            log::warn!(
                "Unknown combination of webview textures: {:?} {:?}",
                texture.format,
                image.texture_descriptor.format
            );

            false
        }
    }
}
