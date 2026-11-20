---
name: glazewm-scratchpad-feature
description: Implementation details, root-cause bug, and design history of GlazeWM's scratchpad feature
metadata:
  type: project
---

Scratchpad feature implemented on branch `feature/scratchpad` in `D:\codes\tes\glazewm`, commits `858cddd6` and `cc1f3a6e`. Part of [[glazewm-i3-features-project]].

**Final design (NOT a literal i3 port — redesigned after user testing):**
Four commands instead of i3's two (`move scratchpad`/`scratchpad show`), because a single overloaded toggle was ambiguous with more than one stashed window:
- `move-to-scratchpad` — hide focused window into an internal, never-displayed scratchpad workspace (not declared in user config; excluded from workspace cycling/queries).
- `toggle-scratchpad` — show/hide the scratchpad's one "current" window.
- `switch-scratchpad-window` — while shown, swap to the next waiting window (round-robin via plain insertion order).
- `remove-from-scratchpad` — tile the shown window into the current workspace, removing it from the pool (originally left it floating; user asked to make it tile instead).

Module: `packages/wm/src/commands/scratchpad/`. New field `is_scratchpad_window: bool` added to both `TilingWindowInner`/`NonTilingWindowInner` (threaded through `WindowGetters` trait + macro + constructors + `to_tiling`/`to_non_tiling` conversions + `manage_window.rs` + `test_utils.rs` mocks).

**Root-cause bug (non-obvious, cost a full debug cycle):** `move_container_within_tree` is NOT a safe generic "move this container" primitive for all cases — it has focus-propagation logic (`is_subtree_focused` check comparing `focus_index()` of the origin vs. target ancestor) designed for ordinary tiling moves, which calls `set_focused_descendant` and reorders the **target parent's** `child_focus_order` based on monitor-level focus history. Since the scratchpad workspace is essentially never focused, this made whichever window was just hidden jump back to the front of the scratchpad's children every time, so only the most-recently-toggled window could ever reappear — a real functional bug, not a flaky test. Fix: use raw `detach_container` + `attach_container` directly instead, which have no such side effect. **Apply this lesson** to any future GlazeWM work that moves a container between two containers that aren't normal sibling tiling positions (e.g. future marks/layout-restore work) — don't reach for `move_container_within_tree` by default; check whether its focus-propagation behavior is actually wanted first.

**Also fixed along the way:** sample config used `alt+minus` which isn't a valid key name — GlazeWM's key parser requires `oem_minus` for the physical `-`/`_` key (`minus`/`subtract` are numpad-only). Invalid key names in a keybinding fail that keybinding's parse silently rather than erroring loudly, which made the first bug report ("alt+minus didn't work") initially look unrelated to the real ordering bug.
