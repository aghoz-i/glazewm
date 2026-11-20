use anyhow::Context;

use super::{
  find_scratchpad_workspace, find_shown_scratchpad_window,
  move_to_scratchpad::move_to_scratchpad,
  show_scratchpad_window::show_scratchpad_window,
};
use crate::{
  traits::CommonGetters, user_config::UserConfig, wm_state::WmState,
};

/// Shows/hides the scratchpad's currently active window.
///
/// - If a scratchpad window is currently shown, it's hidden (moved back to
///   the scratchpad).
/// - Otherwise, the most recently stashed scratchpad window is shown as a
///   floating window on the focused workspace. If the scratchpad is empty,
///   this is a no-op.
pub fn toggle_scratchpad(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if let Some(shown_window) = find_shown_scratchpad_window(state) {
    return move_to_scratchpad(shown_window, state, config);
  }

  let Some(scratchpad_workspace) = find_scratchpad_workspace(state) else {
    return Ok(());
  };

  let Some(window) = scratchpad_workspace
    .children()
    .back()
    .and_then(|container| container.as_window_container().ok())
  else {
    return Ok(());
  };

  let target_workspace = state
    .focused_container()
    .and_then(|container| container.workspace())
    .context("No focused workspace.")?;

  show_scratchpad_window(window, &target_workspace, state, config)
}
