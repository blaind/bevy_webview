//! This module is based on bevy_ui, with added positions for Interaction
use bevy::{core::FloatOrd, input::ElementState, prelude::*, ui::FocusPolicy};

use crate::{
    types::{EventTransport, WebviewAction},
    Webview,
};

#[derive(Component, Copy, Clone, Debug)]
pub enum WebviewInteraction {
    /// The node has been clicked
    Clicked(Vec2),
    /// The node has been hovered over
    Hovered(Vec2),
    /// Nothing has happened
    None,
}

impl Default for WebviewInteraction {
    fn default() -> Self {
        Self::None
    }
}

/// Contains entities whose Interaction should be set to None
#[derive(Default)]
pub struct State {
    entities_to_reset: Vec<Entity>,
}

// based on bevy_ui
pub(crate) fn webview_ui_focus_system(
    mut state: Local<State>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    mut node_query: Query<
        (
            Entity,
            &Node,
            &GlobalTransform,
            Option<&mut WebviewInteraction>,
            Option<&FocusPolicy>,
            Option<&CalculatedClip>,
        ),
        With<Webview>,
    >,
    event_transport: ResMut<EventTransport>,
) {
    let cursor_position = if let Some(cursor_position) = windows
        .get_primary()
        .and_then(|window| window.cursor_position())
    {
        cursor_position
    } else {
        return;
    };

    // reset entities that were both clicked and released in the last frame
    for entity in state.entities_to_reset.drain(..) {
        if let Ok(mut interaction) = node_query.get_component_mut::<Interaction>(entity) {
            *interaction = Interaction::None;
        }
    }

    let mouse_released =
        mouse_button_input.just_released(MouseButton::Left) || touches_input.just_released(0);

    let _right_released = mouse_button_input.just_released(MouseButton::Right);

    if mouse_released {
        for (entity, _node, _global_transform, interaction, _focus_policy, _clip) in
            node_query.iter_mut()
        {
            if let Some(mut interaction) = interaction {
                if let WebviewInteraction::Clicked(offset) = *interaction {
                    event_transport
                        .webview_action_tx
                        .send(WebviewAction::Click((
                            entity,
                            MouseButton::Left,
                            ElementState::Released,
                            offset, // FIXME : currently using wrong offset
                        )))
                        .unwrap();

                    *interaction = WebviewInteraction::None;
                }
            }
        }
    }

    let mouse_clicked =
        mouse_button_input.just_pressed(MouseButton::Left) || touches_input.just_released(0);

    let right_clicked = mouse_button_input.just_pressed(MouseButton::Right);

    let mut moused_over_z_sorted_nodes = node_query
        .iter_mut()
        .filter_map(
            |(entity, node, global_transform, interaction, focus_policy, clip)| {
                let position = global_transform.translation;
                let ui_position = position.truncate();
                let extents = node.size / 2.0;
                let mut min = ui_position - extents;
                let mut max = ui_position + extents;
                if let Some(clip) = clip {
                    min = Vec2::max(min, clip.clip.min);
                    max = Vec2::min(max, clip.clip.max);
                }
                // if the current cursor position is within the bounds of the node, consider it for
                // clicking
                if (min.x..max.x).contains(&cursor_position.x)
                    && (min.y..max.y).contains(&cursor_position.y)
                {
                    let offset = Vec2::new(
                        (cursor_position.x - min.x) / (max.x - min.x),
                        (cursor_position.y - min.y) / (max.y - min.y),
                    );

                    Some((
                        entity,
                        focus_policy,
                        interaction,
                        FloatOrd(position.z),
                        offset,
                    ))
                } else {
                    if let Some(mut interaction) = interaction {
                        if let WebviewInteraction::Hovered(_) = *interaction {
                            *interaction = WebviewInteraction::None;
                        }
                    }
                    None
                }
            },
        )
        .collect::<Vec<_>>();

    moused_over_z_sorted_nodes.sort_by_key(|(_, _, _, z, _)| -*z);
    let mut moused_over_z_sorted_nodes = moused_over_z_sorted_nodes.into_iter();

    // set Clicked or Hovered on top nodes
    for (entity, focus_policy, interaction, _, offset) in moused_over_z_sorted_nodes.by_ref() {
        event_transport
            .webview_action_tx
            .send(WebviewAction::MouseMotion((entity, offset)))
            .unwrap();

        if let Some(mut interaction) = interaction {
            if mouse_clicked {
                event_transport
                    .webview_action_tx
                    .send(WebviewAction::Click((
                        entity,
                        MouseButton::Left,
                        ElementState::Pressed,
                        offset,
                    )))
                    .unwrap();
            }

            if right_clicked {
                event_transport
                    .webview_action_tx
                    .send(WebviewAction::Click((
                        entity,
                        MouseButton::Right,
                        ElementState::Pressed,
                        offset,
                    )))
                    .unwrap();
            }

            if mouse_clicked {
                // only consider nodes with Interaction "clickable"
                if let WebviewInteraction::Clicked(_) = *interaction {
                } else {
                    *interaction = WebviewInteraction::Clicked(offset);
                    // if the mouse was simultaneously released, reset this Interaction in the next
                    // frame
                    if mouse_released {
                        state.entities_to_reset.push(entity);
                    }
                }
            } else if let WebviewInteraction::None = *interaction {
                *interaction = WebviewInteraction::Hovered(offset);
            } else if let WebviewInteraction::Hovered(_) = *interaction {
                *interaction = WebviewInteraction::Hovered(offset);
            }
        }

        match focus_policy.cloned().unwrap_or(FocusPolicy::Block) {
            FocusPolicy::Block => {
                break;
            }
            FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
        }
    }
    // reset lower nodes to None
    for (_entity, _focus_policy, interaction, _, _) in moused_over_z_sorted_nodes {
        if let Some(mut interaction) = interaction {
            if let WebviewInteraction::None = *interaction {
                *interaction = WebviewInteraction::None;
            }
        }
    }
}
