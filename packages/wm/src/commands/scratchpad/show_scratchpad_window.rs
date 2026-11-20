use wm_common::WindowState;

use crate::{
  commands::{
    container::{attach_container, detach_container, set_focused_descendant},
    window::{set_window_position, update_window_state, WindowPositionTarget},
  },
  models::{Workspace, WindowContainer},
  traits::WindowGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Shows a window from the scratchpad on the given workspace as a
/// floating, focused window.
///
/// Assumes `window` is currently a child of the scratchpad workspace.
pub fn show_scratchpad_window(
  window: WindowContainer,
  target_workspace: &Workspace,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  detach_container(window.clone().into())?;
  attach_container(&window.clone().into(), &target_workspace.clone().into(), None)?;

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

  set_focused_descendant(&window.into(), None);
  state.pending_sync.queue_focus_change();

  Ok(())
}
