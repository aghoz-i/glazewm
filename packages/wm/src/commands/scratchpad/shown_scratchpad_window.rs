use crate::{
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

/// Gets the scratchpad window that's currently shown (if any).
///
/// There's at most one at a time - `toggle_scratchpad`/
/// `switch_scratchpad_window` always hide the previously shown window
/// before showing another one.
pub fn find_shown_scratchpad_window(state: &WmState) -> Option<WindowContainer> {
  state.windows().into_iter().find(|window| {
    window.is_scratchpad_window()
      && window
        .workspace()
        .is_some_and(|workspace| !workspace.is_scratchpad())
  })
}
