use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

/// Whether a direction container's children divide its space (`Tiling`) or
/// fully overlap, cascading out from the focused child (`Accordion`).
///
/// Orthogonal to [`crate::TilingDirection`], which only ever describes
/// orientation. An accordion container's `TilingDirection` still determines
/// whether it peeks sideways (reads like tabs) or downward (reads like a
/// stack) - there's no separate "tabbed" vs "stacked" variant here.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerLayout {
  #[default]
  Tiling,
  Accordion,
}

impl FromStr for ContainerLayout {
  type Err = anyhow::Error;

  /// Parses a string into a container layout.
  ///
  /// Example:
  /// ```
  /// # use wm_common::ContainerLayout;
  /// # use std::str::FromStr;
  /// let layout = ContainerLayout::from_str("tiling");
  /// assert_eq!(layout.unwrap(), ContainerLayout::Tiling);
  ///
  /// let layout = ContainerLayout::from_str("accordion");
  /// assert_eq!(layout.unwrap(), ContainerLayout::Accordion);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "tiling" => Ok(Self::Tiling),
      "accordion" => Ok(Self::Accordion),
      _ => bail!("Not a valid container layout: {}", unparsed),
    }
  }
}
