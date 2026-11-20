use anyhow::Context;
use tracing::info;
use wm_common::WindowState;

use super::{
  find_scratchpad_workspace, move_to_scratchpad::move_to_scratchpad,
};
use crate::{
  commands::{
    container::{attach_container, detach_container, set_focused_descendant},
    window::{set_window_position, update_window_state, WindowPositionTarget},
  },
  models::Container,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Shows/hides a window from the scratchpad.
///
/// Equivalent to i3's `scratchpad show`:
/// - If the subject container is a window that's currently shown from the
///   scratchpad, it's hidden again (same as `move_to_scratchpad`).
/// - Otherwise, the most recently stashed scratchpad window is shown as a
///   floating window on the subject's workspace and focused. If the
///   scratchpad is empty, this is a no-op.
pub fn toggle_scratchpad(
  subject_container: &Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if let Ok(window) = subject_container.as_window_container() {
    let is_currently_shown = window.is_scratchpad_window()
      && window
        .workspace()
        .is_some_and(|workspace| !workspace.is_scratchpad());

    if is_currently_shown {
      return move_to_scratchpad(window, state, config);
    }
  }

  let Some(scratchpad_workspace) = find_scratchpad_workspace(state) else {
    return Ok(());
  };

  // Get the most recently stashed scratchpad window.
  let Some(window) = scratchpad_workspace
    .descendant_focus_order()
    .find_map(|descendant| descendant.as_window_container().ok())
  else {
    return Ok(());
  };

  let target_workspace = subject_container
    .workspace()
    .context("Subject container has no workspace.")?;

  info!("Showing window from scratchpad.");

  detach_container(window.clone().into())?;
  attach_container(
    &window.clone().into(),
    &target_workspace.clone().into(),
    None,
  )?;

  // Queue a redraw unconditionally, since the window is transitioning from
  // hidden (scratchpad) to shown and needs to be re-evaluated on the next
  // sync regardless of whether its floating config/placement end up
  // unchanged below.
  state.pending_sync.queue_container_to_redraw(window.clone());

  let floating_defaults =
    &config.value.window_behavior.state_defaults.floating;

  let window = update_window_state(
    window,
    WindowState::Floating(floating_defaults.clone()),
    state,
    config,
  )?;

  if !window.has_custom_floating_placement() {
    set_window_position(
      window.clone(),
      &WindowPositionTarget::Centered,
      state,
    )?;
  }

  set_focused_descendant(&window.clone().into(), None);
  state.pending_sync.queue_focus_change();

  Ok(())
}
