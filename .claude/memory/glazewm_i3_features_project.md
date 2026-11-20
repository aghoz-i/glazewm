---
name: glazewm-i3-features-project
description: Ongoing initiative to add i3-style features to a GlazeWM fork at D:\codes\tes\glazewm
metadata:
  type: project
---

User is adding i3-style features to GlazeWM (github.com/glzr-io/glazewm, Rust, GPL-3.0), cloned to `D:\codes\tes\glazewm`. Goal: mergeable-upstream PRs, one feature per branch/commit set, not a permanent fork.

Five target features (checked against upstream on 2026-11-21, none existed, none in progress except tabbed/stacked):
1. **Scratchpad** — DONE, see [[glazewm-scratchpad-feature]].
2. **Marks** — not started. Nothing found upstream (no issue, no implementation).
3. **focus parent/focus child** — not started. Harder than it looks: GlazeWM's `focused_container()` always resolves to a leaf (window or empty workspace) via `child_focus_order` walk — there's no notion of a non-leaf "selected" container. Needs a model change, not just a new command.
4. **Tabbed/stacked layout** — not started by user. Upstream: PR #1410 (accordion layout) closed unreviewed/unexplained; near-duplicate PR #1459 open unreviewed as of 2026-11-21. Heavily requested (issues #233 46 reactions, #1005 56 reactions). Worth checking PR #1459's status before starting, to avoid duplicate work.
5. **Layout save/restore** — not started. DTO layer (`to_dto()` on every container) already gives a tree snapshot for "save"; "restore" (matching windows to saved slots) has no upstream precedent.

**Build setup on this machine:** Rust nightly installed via rustup, VS Build Tools (C++ workload) installed via winget — neither was present initially. Binary built to `target/debug/glazewm.exe`. User tests by running `.\target\debug\glazewm.exe start --config="resources\assets\sample-config.yaml"` manually and pasting back `RUST_LOG`-style stdout.

**Why:** User wants to keep a small set of focused PRs. Decisions on exact behavior get made interactively with the user via rapid build→test→log→fix cycles rather than upfront spec.

**How to apply:** When resuming work on any of the remaining 4 features, re-verify upstream status first (PRs/issues move fast on this repo). Follow the same workflow as scratchpad: read the relevant model/command files first, propose a design, get approval, implement on its own branch, build+clippy clean, let the user manually test and report back via pasted logs.
