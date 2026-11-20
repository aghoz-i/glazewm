use wm_common::WindowState;

use super::find_shown_scratchpad_window;
use crate::{
  commands::window::update_window_state, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

/// Removes the currently shown scratchpad window from the scratchpad pool,
/// tiling it into the workspace it's currently shown on.
///
/// A no-op unless a scratchpad window is currently shown.
pub fn remove_from_scratchpad(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if let Some(shown_window) = find_shown_scratchpad_window(state) {
    let window =
      update_window_state(shown_window, WindowState::Tiling, state, config)?;

    window.set_is_scratchpad_window(false);
  }

  Ok(())
}
