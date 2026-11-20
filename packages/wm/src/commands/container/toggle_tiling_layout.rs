use anyhow::Context;
use wm_common::{ContainerLayout, WmEvent};

use crate::{
  models::Container,
  traits::{CommonGetters, ContainerLayoutGetters},
  wm_state::WmState,
};

/// Toggles a container between the normal `Tiling` layout and `Accordion`
/// layout.
///
/// Unlike `toggle_tiling_direction`, this never needs to wrap the subject
/// in a new split container - the layout mode lives on the direction
/// container itself, so it can be flipped in place regardless of how many
/// children it currently has.
pub fn toggle_tiling_layout(
  container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let direction_container = container
    .direction_container()
    .context("No direction container.")?;

  let new_layout = match direction_container.container_layout() {
    ContainerLayout::Tiling => ContainerLayout::Accordion,
    ContainerLayout::Accordion => ContainerLayout::Tiling,
  };

  direction_container.set_container_layout(new_layout);

  state
    .pending_sync
    .queue_containers_to_redraw(direction_container.tiling_children());

  state.emit_event(WmEvent::TilingLayoutChanged {
    direction_container: direction_container.to_dto()?,
    new_container_layout: direction_container.container_layout(),
  });

  Ok(())
}
