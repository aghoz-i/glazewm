use tracing::info;
use wm_common::WindowState;

use super::scratchpad_workspace::scratchpad_workspace;
use crate::{
  commands::{
    container::{move_container_within_tree, set_focused_descendant},
    window::update_window_state,
  },
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Moves a window to the scratchpad, hiding it.
///
/// Equivalent to i3's `move scratchpad`. If the window is currently tiling,
/// it's converted to floating first, since scratchpad windows are always
/// shown as floating.
pub fn move_to_scratchpad(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Scratchpad windows are always floating when shown again, so convert
  // now while the window still has a valid workspace to float within.
  let window = match window {
    WindowContainer::TilingWindow(_) => {
      let floating_defaults =
        &config.value.window_behavior.state_defaults.floating;

      update_window_state(
        window,
        WindowState::Floating(floating_defaults.clone()),
        state,
        config,
      )?
    }
    WindowContainer::NonTilingWindow(_) => window,
  };

  info!("Moving window to scratchpad.");

  // Get focus target before detaching, since it depends on the window's
  // current workspace.
  let focus_target = state.focus_target_after_removal(&window);

  let scratchpad_workspace = scratchpad_workspace(state)?;

  move_container_within_tree(
    &window.clone().into(),
    &scratchpad_workspace.clone().into(),
    scratchpad_workspace.child_count(),
    state,
  )?;

  window.set_is_scratchpad_window(true);

  if let Some(focus_target) = focus_target {
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change();
  }

  state.pending_sync.queue_container_to_redraw(window);

  Ok(())
}
