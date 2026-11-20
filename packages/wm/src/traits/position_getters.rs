use ambassador::delegatable_trait;
use uuid::Uuid;
use wm_common::TilingDirection;
use wm_platform::Rect;

use crate::{
  models::DirectionContainer,
  traits::{CommonGetters, TilingDirectionGetters},
};

#[delegatable_trait]
pub trait PositionGetters {
  fn to_rect(&self) -> anyhow::Result<Rect>;
}

/// Gets the rect for a child of an accordion-layout container.
///
/// Every child (including the focused one) is the same size, cascading
/// from the container's edge in tree order so the whole stack always fits
/// within the space a single tiling window would occupy there - never
/// overflowing into siblings or, for a top-level accordion, off the
/// screen. If the configured padding would make the cascade wider/taller
/// than the container itself, it's shrunk dynamically so every window
/// still fits. Which child renders on top is handled separately, by
/// z-order (see `accordion_z_order` in `platform_sync.rs`).
pub fn accordion_child_rect(
  parent: &DirectionContainer,
  parent_rect: &Rect,
  self_id: Uuid,
  accordion_paddings: (i32, i32),
) -> Rect {
  let tiling_children = parent.tiling_children().collect::<Vec<_>>();

  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let self_index = tiling_children
    .iter()
    .position(|child| child.id() == self_id)
    .unwrap_or(0) as i32;

  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let total_children = tiling_children.len() as i32;

  let cascade = |dimension: i32, padding: i32| {
    let max_total_offset = (total_children - 1) * padding;

    let safe_padding = if total_children > 1 && max_total_offset >= dimension
    {
      (dimension / total_children.max(2)).max(1)
    } else {
      padding
    };

    let size = (dimension - (total_children - 1) * safe_padding).max(1);

    (self_index * safe_padding, size)
  };

  let (horizontal_padding, vertical_padding) = accordion_paddings;

  match parent.tiling_direction() {
    TilingDirection::Horizontal => {
      let (x_offset, width) = cascade(parent_rect.width(), horizontal_padding);

      Rect::from_xy(
        parent_rect.x() + x_offset,
        parent_rect.y(),
        width,
        parent_rect.height(),
      )
    }
    TilingDirection::Vertical => {
      let (y_offset, height) = cascade(parent_rect.height(), vertical_padding);

      Rect::from_xy(
        parent_rect.x(),
        parent_rect.y() + y_offset,
        parent_rect.width(),
        height,
      )
    }
  }
}

/// Implements the `PositionGetters` trait for tiling containers that can
/// be resized. This is used by `SplitContainer` and `TilingWindow`.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_getters_as_resizable {
  ($struct_name:ident) => {
    impl PositionGetters for $struct_name {
      fn to_rect(&self) -> anyhow::Result<Rect> {
        let parent = self
          .parent()
          .and_then(|parent| parent.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let parent_rect = parent.to_rect()?;

        if parent.container_layout() == ContainerLayout::Accordion {
          return Ok($crate::traits::accordion_child_rect(
            &parent,
            &parent_rect,
            self.id(),
            self.accordion_paddings()?,
          ));
        }

        let (horizontal_gap, vertical_gap) = self.inner_gaps()?;
        let inner_gap = match parent.tiling_direction() {
          TilingDirection::Vertical => vertical_gap,
          TilingDirection::Horizontal => horizontal_gap,
        };

        #[allow(
          clippy::cast_precision_loss,
          clippy::cast_possible_truncation,
          clippy::cast_possible_wrap
        )]
        let (width, height) = match parent.tiling_direction() {
          TilingDirection::Vertical => {
            let available_height = parent_rect.height()
              - inner_gap * self.tiling_siblings().count() as i32;

            let height =
              (self.tiling_size() * available_height as f32) as i32;

            (parent_rect.width(), height)
          }
          TilingDirection::Horizontal => {
            let available_width = parent_rect.width()
              - inner_gap * self.tiling_siblings().count() as i32;

            let width =
              (available_width as f32 * self.tiling_size()).round() as i32;

            (width, parent_rect.height())
          }
        };

        let (x, y) = {
          let mut prev_siblings = self
            .prev_siblings()
            .filter_map(|sibling| sibling.as_tiling_container().ok());

          match prev_siblings.next() {
            None => (parent_rect.x(), parent_rect.y()),
            Some(sibling) => {
              let sibling_rect = sibling.to_rect()?;

              match parent.tiling_direction() {
                TilingDirection::Vertical => (
                  parent_rect.x(),
                  sibling_rect.y() + sibling_rect.height() + inner_gap,
                ),
                TilingDirection::Horizontal => (
                  sibling_rect.x() + sibling_rect.width() + inner_gap,
                  parent_rect.y(),
                ),
              }
            }
          }
        };

        Ok(Rect::from_xy(x, y, width, height))
      }
    }
  };
}
