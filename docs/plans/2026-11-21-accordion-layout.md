# Accordion tiling layout — implementation plan

Status: **implemented (commit `c1e4c811`), geometry revised after first
round of manual testing (see "Post-implementation revision" below)**
Branch: `feature/accordion-layout`

## Background

GlazeWM is missing i3-style tabbed/stacked layouts (upstream issues
[#233](https://github.com/glzr-io/glazewm/issues/233) — 46 reactions,
[#1005](https://github.com/glzr-io/glazewm/issues/1005) — 56 reactions, marked
"in progress"). Two upstream attempts exist, both unreviewed as of
2026-11-21:

- PR [#1410](https://github.com/glzr-io/glazewm/pull/1410) — closed, unreviewed,
  no explanation given.
- `origin/feat/accordian-v1` (remote branch, 6 commits, never opened as a PR,
  abandoned/WIP) — a third-party contributor's personal attempt at this exact
  feature, pushed directly to the origin remote. Reviewed its diffs on
  2026-11-21 (see "Lessons from `feat/accordian-v1`" below): it extends
  `TilingDirection` itself with `HorizontalAccordion`/`VerticalAccordion`
  variants, which forced ~200 lines of special-casing into
  `focus_in_direction.rs`/`move_window_in_direction.rs` and a reactive
  "temp z-order hack" commit. Confirms the orthogonal-property design
  decision below avoids that entire class of problem, and surfaced a real
  z-order gap (see decision 7) that the original plan missed.
- PR [#1459](https://github.com/glzr-io/glazewm/pull/1459) — open, no
  maintainer review yet. Adds `set-tiling-layout`/`toggle-tiling-layout`
  commands and a horizontal/vertical "accordion" geometry.

Neither implements a real clickable tab bar — GlazeWM draws **zero** custom
window chrome today (no title bars, no decorations of its own), so a literal
i3-style tab strip would require a brand-new always-on-top overlay window
subsystem in `wm-platform` (hit-testing, redraw-on-resize, DPI handling) with
no existing code to build from. Both upstream PRs instead use an "accordion"
effect: windows overlap almost entirely, with the focused one full-size on
top and neighbors peeking out at the edge.

## Decisions (made with user, 2026-11-21)

1. **Visual approach: Accordion peek**, not a real tab bar. No title bar, no
   mouse click-to-switch. Matches the unreviewed upstream PRs' approach and
   reuses existing position/z-order code with no new Win32 subsystem.
2. **Data model: separate orthogonal property**, not new `TilingDirection`
   variants. `TilingDirection` keeps its existing meaning (orientation only);
   a new `ContainerLayout` (`Tiling` | `Accordion`) controls whether children
   divide space or overlap.
3. **Scope: per split-container** (nestable), not workspace-only — matches
   real i3, where any container (including a sub-group) can be tabbed/stacked.
4. **No separate "tabbed" vs "stacked" concepts.** Since there's no title bar
   to visually distinguish them, it's one mechanism (`Accordion`) whose
   peek-direction is simply whatever `TilingDirection` the container already
   has (horizontal accordion reads like tabs, vertical reads like a stack).
5. **Directional focus (arrow keys) cycles tabs** when inside an accordion
   container, matching i3. Confirmed this requires **no code changes** —
   `focus_in_direction.rs`'s `tiling_focus_target` already walks tree-order
   siblings whenever the parent's `TilingDirection` matches the key's axis; it
   never did real geometric comparison. No wraparound at the ends (confirmed
   against user's recollection of real i3 behavior) — pressing past the last
   tab falls through to whatever's outside the container, or stops if
   there's nothing there. This is the existing, unmodified behavior.
6. **`move --direction` inside an accordion container** also requires **no
   code changes**, confirmed by tracing `move_window_in_direction.rs`:
   - Not at the end of the stack: swaps tree position with the adjacent tab
     via `move_to_sibling_container`/`move_container_within_tree` — pure tree
     reordering, no geometry involved.
   - At the end: falls through to `insert_into_ancestor`, exiting the
     container — already generic, no accordion-awareness needed.
   - Accordion container reduced from 2 children to 1 after a tab moves out:
     `move_container_within_tree` always finishes by calling
     `flatten_child_split_containers` on the moved window's original parent/
     grandparent/great-grandparent. That function already unconditionally
     dissolves any same-direction split container down to one remaining
     child, splicing it up as a plain sibling — it doesn't check or care
     whether the dissolved container was `Accordion` or `Tiling`. This is
     the existing "redundant split container" cleanup
     (`H[1 H[2,3]] -> H[1,2,3]`), already present and generic.
   - **Important constraint on implementation**: don't touch
     `move_window_in_direction.rs`, `move_container_within_tree.rs`, or
     `flatten_child_split_containers.rs` at all — this behavior falls out for
     free, same as directional focus.
7. **Z-order — CORRECTED on 2026-11-21 after reviewing `feat/accordian-v1`.**
   Original plan claimed "no changes needed," reasoning that
   `platform_sync.rs::redraw_containers` already computes "bring to front"
   from `workspace.descendant_focus_order().next()` and stacks every other
   window with `WindowZOrder::AfterWindow(focused.id())`. That's true, but
   incomplete: it only guarantees the globally-focused window is in front of
   *everything else* — it says nothing about the correct relative order
   *among* the non-focused siblings. For exactly 2 siblings that's fine
   (there's only one "other"), but for 3+ accordion siblings, every
   non-focused window independently gets told "go right behind the focused
   one," which in practice orders them by focus-recency history, not by
   peek-distance from the focused child. A sibling that should peek far
   behind (large `|offset|`) could end up rendered in front of one that
   should peek closer. The abandoned `feat/accordian-v1` branch hit exactly
   this and bolted on a reactive "temp z-order hack" (a second pass forcing
   the focused window to `ZOrder::Top`, which doesn't even fix the ordering
   *among* the other siblings).
   **Fix**: add a dedicated pass in `redraw_containers` for each accordion
   `DirectionContainer`: collect its tiling children, sort by ascending
   `|sibling_index - focused_index|`, then chain their z-order so each one
   (other than the focused child) is placed `AfterWindow` of the previous
   one in that sorted sequence. This guarantees the visible stack always
   layers correctly by peek-distance regardless of focus history. Since
   accordion children never leave the `Shown`/`Hidden` display-state
   machinery (confirmed still correct — see above), this pass only needs to
   touch z-order, not position/display-state.
8. **Resize commands** (`resize --width/--height`) inside an accordion
   container are a no-op by construction — they adjust `tiling_size`, which
   the new accordion geometry branch never reads. No special-casing needed.

## Data model changes

- **`packages/wm-common/src/container_layout.rs` (new)**:
  `pub enum ContainerLayout { Tiling, Accordion }`, `Default` = `Tiling`,
  serde support, mirroring `tiling_direction.rs`'s structure.
- **`packages/wm/src/models/workspace.rs`, `split_container.rs`**: add a
  `container_layout: ContainerLayout` field next to the existing
  `tiling_direction` field.
- **New trait `ContainerLayoutGetters`** (`packages/wm/src/traits/`) with a
  macro `impl_container_layout_getters!`, following the exact pattern of
  `impl_tiling_direction_getters!`. Delegated onto `Container`/
  `DirectionContainer` via `ambassador::Delegate`.
- **`packages/wm-common/src/dtos/workspace_dto.rs`,
  `split_container_dto.rs`**: add `container_layout` field so IPC/Zebar
  consumers can see the current layout mode (state exposure only — no
  rendering needed from GlazeWM itself).

## Geometry — `packages/wm/src/traits/position_getters.rs`

`impl_position_getters_as_resizable!`'s `to_rect()` gets a new branch keyed
on `parent.container_layout()`:

- **`Tiling`** (existing): unchanged — proportional space-splitting math as
  today.
- **`Accordion`**: see "Post-implementation revision" below for the
  geometry actually shipped. `tiling_size` is ignored in this branch (not
  read, not cleared) either way.

### Post-implementation revision (2026-11-21, after first manual test)

First version made every child the *same size as the whole accordion
container*, then shifted it by `offset * accordion_padding` from the
focused child. User feedback after testing: this let the stack overflow
past the accordion container's own bounds (into sibling space, or off the
monitor edge for a top-level accordion) once there were enough windows or
large enough offsets — the container's rect should represent "the space
the window that's supposed to be there would occupy," and nothing should
spill outside it.

Revised to a uniform-shrink cascade (confirmed with user: uniform shrink
over "focused stays full-size, only neighbors shrink," and dynamic
padding shrink over capping visible peeks): every child, including the
focused one, is sized `container_dimension - (N-1) * padding` along the
accordion's axis, positioned at `tree_index * padding` from the
container's edge — so the whole cascade always fits exactly within the
container's own rect, regardless of N. If `(N-1) * padding` would exceed
the container's dimension, padding is dynamically shrunk
(`container_dimension / N`, floored at 1px) so it still fits however many
windows there are, rather than capping how many peek and hiding the rest.

This also simplified the implementation: position no longer needs to know
which child is focused at all (that's purely a z-order concern, handled
separately — see decision 7's correction above) — it's now pure tree-index
cascade math. Extracted into a standalone function,
`accordion_child_rect` in `position_getters.rs`, called via a `$crate::`
path from the macro (which otherwise hit clippy's `too_many_lines`).

## New commands

- **`packages/wm-common/src/app_command.rs`**:
  - `InvokeCommand::ToggleTilingLayout` — toggles the focused container's/
    workspace's `ContainerLayout` between `Tiling`/`Accordion`, mirroring
    `toggle-tiling-direction`'s existing shape.
  - `InvokeCommand::SetTilingLayout { layout: ContainerLayout }` — explicit
    set, mirroring `set-tiling-direction`.
- **New module `packages/wm/src/commands/container/toggle_tiling_layout.rs`
  (+ `set_tiling_layout.rs`)**, following `toggle_tiling_direction.rs`'s
  exact structure: operates on the focused window's `direction_container()`
  (or the workspace if a lone child), flips/sets `container_layout`, emits a
  new `WmEvent::TilingLayoutChanged` (mirroring `TilingDirectionChanged`),
  queues a redraw of the container's tiling children.
- Wire into `wm.rs::run_command`.

## Config

- **`packages/wm-common/src/parsed_config.rs`**: add
  `accordion_padding: LengthValue` to `GapsConfig` (default 30px, matching
  the open upstream PR's naming).
- **`resources/assets/sample-config.yaml`**: document the new
  `gaps.accordion_padding` option, and add example keybindings (e.g.
  `alt+w` → `toggle-tiling-layout`, matching the existing `alt+v` →
  `toggle-tiling-direction` pattern).

## Explicitly out of scope for this PR

- No visual tab bar / title strip of any kind (no title-bar chrome exists in
  GlazeWM to build on).
- No mouse click-to-switch-tab (no overlay window, so nothing to click).
- No distinction between "tabbed" and "stacked" as separate concepts — just
  accordion + whatever `TilingDirection` is already set.

## Files touched (summary)

**`wm-common`**: `container_layout.rs` (new), `app_command.rs`,
`parsed_config.rs`, `wm_event.rs`, `dtos/workspace_dto.rs`,
`dtos/split_container_dto.rs`

**`wm`**: `traits/container_layout_getters.rs` (new), `models/workspace.rs`,
`models/split_container.rs`, `traits/position_getters.rs`,
`commands/container/toggle_tiling_layout.rs` (new),
`commands/container/set_tiling_layout.rs` (new),
`commands/container/mod.rs`, `wm.rs`,
`commands/general/platform_sync.rs` (new accordion z-order ordering pass —
see decision 7's correction)

**Explicitly NOT touched**: `move_window_in_direction.rs`,
`move_container_within_tree.rs`, `flatten_child_split_containers.rs`,
`focus_in_direction.rs` — all confirmed to already produce correct accordion
behavior unmodified.

`resources/assets/sample-config.yaml`
