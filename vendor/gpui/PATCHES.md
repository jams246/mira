# MIRA native text worker patch

Based on the unmodified crates.io `gpui` 0.2.2 package, licensed under Apache 2.0.
The original package source is retained here so builds do not depend on changes
to a developer's Cargo registry cache.

Local changes are limited to these existing source files:

- `src/platform/windows/direct_write.rs`: isolate expensive native layout from
  shared font state; preserve exact fallback font identity when publishing.
- `src/text_system.rs`: provide independent worker caches and immutable layout
  snapshots; keep font shaping independent of paint decorations.
- `src/text_system/line_layout.rs`: snapshot and retain completed cached layouts
  without sharing a live frame cache with worker threads.

MIRA owns task scheduling, document cancellation and retained-layout lifetime.
Native glyph placement, text, font features, wrapping and hit testing remain
DirectWrite and GPUI operations. No alternate shaping engine is introduced.

Performance and native rendering evidence is recorded locally in
`.verification/unicode-layout-stall/`. Upstream adoption of equivalent worker
support would allow removal of this patch.
