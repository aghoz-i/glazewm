use ambassador::delegatable_trait;
use wm_common::ContainerLayout;

use super::CommonGetters;

#[delegatable_trait]
pub trait ContainerLayoutGetters: CommonGetters {
  fn container_layout(&self) -> ContainerLayout;

  fn set_container_layout(&self, container_layout: ContainerLayout);
}

/// Implements the `ContainerLayoutGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `container_layout` field.
#[macro_export]
macro_rules! impl_container_layout_getters {
  ($struct_name:ident) => {
    impl ContainerLayoutGetters for $struct_name {
      fn container_layout(&self) -> ContainerLayout {
        self.0.borrow().container_layout.clone()
      }

      fn set_container_layout(&self, container_layout: ContainerLayout) {
        self.0.borrow_mut().container_layout = container_layout;
      }
    }
  };
}
