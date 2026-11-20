use anyhow::Context;
use wm_common::{GapsConfig, TilingDirection, WorkspaceConfig};

use crate::{
  commands::container::attach_container,
  models::{Workspace, SCRATCHPAD_WORKSPACE_NAME},
  wm_state::WmState,
};

/// Gets the internal scratchpad workspace, if it has been created.
///
/// Returns `None` if nothing has ever been moved to the scratchpad yet.
pub fn find_scratchpad_workspace(state: &WmState) -> Option<Workspace> {
  state
    .workspaces()
    .into_iter()
    .find(Workspace::is_scratchpad)
}

/// Gets the internal scratchpad workspace, creating it if it doesn't
/// already exist.
///
/// This workspace is never declared in the user's config. It's attached to
/// the first available monitor purely so it fits the container tree's
/// invariant that workspaces have a parent monitor - it's never displayed,
/// so which monitor it's attached to has no visible effect.
pub fn scratchpad_workspace(
  state: &mut WmState,
) -> anyhow::Result<Workspace> {
  if let Some(workspace) = find_scratchpad_workspace(state) {
    return Ok(workspace);
  }

  let monitor = state
    .monitors()
    .into_iter()
    .next()
    .context("Cannot create scratchpad workspace with no monitors.")?;

  let workspace = Workspace::new(
    WorkspaceConfig {
      name: SCRATCHPAD_WORKSPACE_NAME.to_string(),
      display_name: None,
      bind_to_monitor: None,
      // Keep the workspace around even when it has no windows stashed in
      // it, so it isn't torn down by empty-workspace cleanup.
      keep_alive: true,
    },
    GapsConfig::default(),
    TilingDirection::Horizontal,
  );

  attach_container(&workspace.clone().into(), &monitor.into(), None)?;

  Ok(workspace)
}
