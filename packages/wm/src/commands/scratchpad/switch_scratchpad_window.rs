use anyhow::Context;

use super::{
  find_scratchpad_workspace, find_shown_scratchpad_window,
  move_to_scratchpad::move_to_scratchpad,
  show_scratchpad_window::show_scratchpad_window,
};
use crate::{traits::CommonGetters, user_config::UserConfig, wm_state::WmState};

/// Hides the currently shown scratchpad window and shows the next one.
///
/// A no-op unless a scratchpad window is currently shown. If no other
/// window is waiting in the scratchpad, the current one stays shown.
pub fn switch_scratchpad_window(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(shown_window) = find_shown_scratchpad_window(state) else {
    return Ok(());
  };

  let Some(scratchpad_workspace) = find_scratchpad_workspace(state) else {
    return Ok(());
  };

  // Nothing else to switch to.
  if scratchpad_workspace.child_count() == 0 {
    return Ok(());
  }

  let target_workspace = shown_window
    .workspace()
    .context("Shown scratchpad window has no workspace.")?;

  move_to_scratchpad(shown_window, state, config)?;

  let Some(next_window) = scratchpad_workspace
    .children()
    .front()
    .and_then(|container| container.as_window_container().ok())
  else {
    return Ok(());
  };

  show_scratchpad_window(next_window, &target_workspace, state, config)
}
