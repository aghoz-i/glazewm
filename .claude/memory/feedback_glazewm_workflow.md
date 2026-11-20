---
name: feedback-glazewm-workflow
description: How this user likes to iterate on GlazeWM feature work (test-driven, pragmatic deviation from i3, terse approvals)
metadata:
  type: feedback
---

When implementing i3-style features for the GlazeWM fork ([[glazewm-i3-features-project]]):

**Pragmatic over faithful.** User explicitly chose to deviate from literal i3 behavior (splitting one overloaded `scratchpad show` toggle into four distinct commands, and making "remove from scratchpad" tile instead of stay floating) once real usage showed the i3-faithful version was confusing with multiple windows. Reason: "a bit different than i3 I guess but its okay" — correctness/usability for their actual workflow beats spec fidelity. How to apply: don't over-anchor on matching i3 exactly once the user has hands-on feedback; propose the behavior change that actually solves what they're hitting, even if it diverges.

**Test loop is manual + log-driven.** User runs the built `.exe` themselves and pastes back full stdout (tracing `INFO`/`WARN` lines) when something's wrong, rather than describing symptoms abstractly. How to apply: read the pasted log closely — the actual bug (e.g. the `move_container_within_tree` focus-reorder) was diagnosable directly from the log's command/event sequence, not from re-reading the implementation in isolation.

**Terse approvals, no re-litigating.** Responses like "nope still the same", "nah just commit", "good commit the changes" — short and directive. How to apply: don't ask multiple clarifying questions once the user has stated a design in their own words (as they did for the 4-command redesign); implement it and verify via build+clippy, then hand back for their own manual test rather than asking "does this look right?" first.

**Rebuild-blocked-by-running-process is a recurring friction point.** `cargo build` fails with "Access is denied" whenever `glazewm.exe` is still running from the user's manual test. How to apply: proactively check/ask before attempting a rebuild if the user was just testing; don't silently retry or kill the process without asking (killing it skips GlazeWM's normal shutdown/window-restore routine).
