# MIRA GPUI patches

Based on the unmodified crates.io `gpui` 0.2.2 package, licensed under Apache 2.0.
The vendored package keeps local changes reproducible without modifying a
developer's Cargo registry cache. Upstream platform implementations and examples
remain available.

## Native text and performance

- `src/platform/windows/direct_write.rs`: isolate expensive native layout from
  shared font state; preserve exact fallback font identity when publishing.
- `src/text_system.rs`: provide independent worker caches and immutable layout
  snapshots; keep font shaping independent of paint decorations.
- `src/text_system/line_layout.rs`: snapshot and retain completed cached layouts
  without sharing a live frame cache with worker threads.
- `src/elements/text.rs`: avoid allocating a line wrapper for unwrapped text.

MIRA owns task scheduling, document cancellation and retained-layout lifetime.
Native glyph placement, text, font features, wrapping and hit testing remain
DirectWrite and GPUI operations. No alternate shaping engine is introduced.

Performance and native rendering evidence is recorded locally in
`.verification/unicode-layout-stall/`. Upstream adoption of equivalent worker
support would allow removal of this patch.

## Windows accessibility and input

- `src/accessibility.rs`, `src/elements/div.rs`, `src/interactive.rs`, and
  `src/window.rs`: collect native roles, names, states, bounds, focus, and actions
  from painted controls; preserve existing input handlers when invoking them.
- `src/platform/windows/accessibility.rs` and its `events`, `nullable`, `tests`,
  and `event_tests` modules: implement Windows UI Automation providers, events,
  nullable COM results, and lifecycle regression coverage.
- `src/platform.rs`, `src/platform/windows.rs`, and Windows `events.rs` and
  `window.rs`: connect native providers and dispatch accessibility requests.
- `src/gpui.rs`: expose the accessibility API used by MIRA.
- `src/interactive.rs` and `src/platform.rs` also expose native input results and
  constructible external paths for MIRA's integration tests.

## Diagnostics and build compatibility

- `src/measurement.rs`: route optional native measurements to MIRA's asynchronous
  diagnostic sink while retaining the upstream fallback.
- `src/taffy.rs`: specify floating-point literal types for the current compiler.
- `Cargo.toml`: enable Windows accessibility APIs and use the maintained
  `stacksafe` macro dependency.

Native cache, fallback, accessibility, and lifecycle tests remain part of the
local patches. MIRA verification artifacts live under `.verification/`;
historical results do not replace checks against the current source.
