use anyhow::Context;
use wm_common::ContainerLayout;

use super::toggle_tiling_layout;
use crate::{
  models::Container,
  traits::{CommonGetters, ContainerLayoutGetters},
  wm_state::WmState,
};

pub fn set_tiling_layout(
  container: &Container,
  state: &mut WmState,
  container_layout: &ContainerLayout,
) -> anyhow::Result<()> {
  let direction_container = container
    .direction_container()
    .context("No direction container.")?;

  if direction_container.container_layout() == *container_layout {
    Ok(())
  } else {
    toggle_tiling_layout(container, state)
  }
}
