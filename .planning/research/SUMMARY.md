# Project Research Summary

**Project:** AMCLI (Apple Music TUI) -- Layout & Visual Polish Milestone
**Domain:** Ratatui TUI layout refactoring and visual polish
**Researched:** 2026-03-26
**Confidence:** HIGH

## Executive Summary

AMCLI is a functioning macOS Apple Music TUI that needs its layout and visual presentation elevated from "works" to "polished." The core problem is a monolithic 455-line `draw()` function using legacy Ratatui patterns (pre-0.30 verbose builders, manual centering arithmetic, percentage-based splits that sum to 99%) that produce uneven spacing, top-left-pinned artwork, and remainder gaps in button distribution. The good news: Ratatui 0.30, already installed, ships every API needed to fix these issues -- `Rect::centered()`, `Constraint::Fill`, `Flex::SpaceBetween`, `Layout::vertical/horizontal()` with `Spacing` -- but the codebase uses none of them. No new dependencies are required.

The recommended approach is a structured decomposition: first establish a spacing system and extract the monolithic draw function into focused section renderers (artwork, metadata, lyrics, progress, controls, chassis), then apply Ratatui 0.30 layout APIs to each section for centering, proportional distribution, and consistent spacing. The extraction must happen before the layout fixes because the current function is too large to safely modify in place -- a change to artwork centering at line 730 can silently break button layout at line 1030. Decomposition isolates blast radius.

The primary risks are borrow checker conflicts when extracting stateful widget calls (artwork and throbber need `&mut` access), ratatui-image re-encoding lag when artwork dimensions change during layout adjustments, and Cassowary constraint solver non-determinism at small terminal sizes. All three have documented prevention strategies: pass narrow field slices instead of `&mut App`, snap artwork dimensions to stable values with resize debouncing, and use `Constraint::Min` for fixed elements with `Constraint::Fill` for flexible areas. The retro theme's visual integrity across all sections is a cross-cutting concern that must be verified at every step.

## Key Findings

### Recommended Stack

No new dependencies. Ratatui 0.30.0 (already installed) provides every API this milestone needs. The `ratatui-macros` crate is already a transitive dependency re-exported via `ratatui::macros`.

**Core APIs to adopt (all available, none currently used):**
- `Layout::vertical()` / `Layout::horizontal()`: Ergonomic layout constructors -- replace verbose `Layout::default().direction(...)` pattern
- `Layout::areas::<N>()`: Compile-time-checked split into `[Rect; N]` -- eliminates runtime index errors
- `Rect::centered()` / `centered_vertically()` / `centered_horizontally()`: Direct centering -- replaces 15+ lines of manual padding math per centering operation
- `Constraint::Fill(weight)`: Proportional space distribution -- replaces `Constraint::Min(0)` and percentage pairs that sum to 99%
- `Flex::SpaceBetween` + `Layout::spacing()`: Even button distribution with gaps -- replaces manual `btn_width = area.width / count` division
- `Spacing::Overlap(1)`: Border sharing between adjacent blocks -- eliminates double-border visual noise

**Critical version note:** `Alignment` was renamed to `HorizontalAlignment` in 0.30. The old name works via type alias but should be migrated in a prep commit to avoid ambiguity with glob imports.

### Expected Features

**Must have (table stakes -- users expect these, absence looks broken):**
- Artwork vertically centered in its area (not pinned top-left)
- Even button distribution across full terminal width (no remainder gap)
- Consistent spacing system across all sections (replace ad-hoc padding values)
- Clean metadata alignment (standardized label/value vertical rhythm)
- Lyrics current-line highlight with sufficient contrast (bold + accent color, not just bold)
- No dead zones at any terminal size (proportional fill, center content in excess space)

**Should have (differentiators -- elevates from "not broken" to "professional"):**
- Adaptive artwork/info split ratio with Min/Max constraints instead of fixed percentages
- Button text truncation at narrow widths (hide key hints first, then use icon-only)
- Separator line between artwork and info columns
- Lyrics edge padding (top/bottom margin so first/last lines are not flush against borders)
- Graduated lyrics dimming (lines near current = slightly dim, lines far away = more dim)

**Defer (v2+):**
- Configurable layout / user-defined panel arrangement (massive scope, rmpc-level effort)
- Smooth scroll interpolation for lyrics (requires fractional scroll position tracking)
- Animated transitions between layout states (not viable at 50ms/20fps TUI refresh)
- Pixel-perfect cross-terminal alignment (impossible due to cell dimension variance)

### Architecture Approach

Decompose the monolithic `draw()` into a layout orchestrator calling free-function section renderers. Each renderer receives a pre-computed `Rect` and narrow data slices (not `&mut App`). Layout decisions (show artwork, column ratios, two-column metadata) stay in the orchestrator. Render order enforces z-ordering (background -> chassis -> content -> overlays).

**Major components:**
1. `render/mod.rs` (orchestrator) -- computes top-level layout Rects, delegates to section renderers in strict z-order
2. `render/chassis.rs` -- retro border chrome, scanlines, screen frame; returns `screen_inner` Rect
3. `render/artwork.rs` -- centering with `Rect::centered()`, three modes: loading/image/no-signal
4. `render/metadata.rs` -- track info labels, single/two-column conditional layout, scroll text
5. `render/lyrics.rs` -- synced lyrics with current-line highlight and graduated dimming
6. `render/progress.rs` -- gauge with time label, fixed 3-row height
7. `render/controls.rs` -- button row with `Flex::SpaceBetween` distribution
8. `render/idle.rs` -- "waiting for media" when no track playing
9. `theme.rs` -- extracted theme constants used by all renderers
10. `helpers.rs` -- pure formatting functions (format_duration, scroll_text)

### Critical Pitfalls

1. **Borrow conflicts on function extraction** -- Extracting stateful widget calls (artwork, throbber) into sub-functions causes E0499/E0502 when passing `&mut App`. Prevention: pass individual field references, not `&mut App`. Design function signatures before writing code.

2. **Cassowary solver non-determinism at small sizes** -- When constraints conflict (terminal under 16 rows), the solver produces arbitrary results. Prevention: use `Constraint::Min` for elements that must appear, `Constraint::Fill` for flexible areas, and add minimum-area guards before every render call.

3. **ratatui-image blocking re-encode on layout change** -- `StatefulProtocol` re-encodes the image whenever the render Rect changes size, causing lag at 50ms frame rate. Prevention: snap artwork dimensions to stable values (round to even numbers), debounce artwork re-render by 200-300ms during terminal resize.

4. **Settings overlay z-order breakage** -- The overlay depends on being rendered absolutely last. Decomposition can accidentally place it before other renderers. Prevention: orchestrator enforces strict render order with overlays guaranteed last.

5. **Retro theme visual integrity** -- The retro theme depends on precise relationships between chassis borders and content areas. Decomposition can break these alignments. Prevention: separate layout computation from rendering, test both themes at every layout change.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 0: Prep Housekeeping
**Rationale:** Mechanical migration that must precede all other work to avoid import ambiguity during refactoring. Low risk, isolates churn from functional changes.
**Delivers:** Clean type imports, ready-to-use 0.30 patterns.
**Addresses:** `Alignment` to `HorizontalAlignment` migration across 8 call sites.
**Avoids:** Pitfall 6 (Alignment migration landmine) -- prevents ambiguity errors that would pollute every subsequent commit.

### Phase 1: Draw Function Decomposition
**Rationale:** The monolithic draw function must be broken apart before layout fixes can be applied safely. Touching centering logic at line 730 in a 455-line function risks silent regressions at line 1030. Decomposition isolates blast radius and is a prerequisite for every subsequent phase.
**Delivers:** Orchestrator + 7 section renderers in `render/` module, extracted `theme.rs` and `helpers.rs`. Functionally identical output -- no visible changes yet.
**Addresses:** Architecture decomposition (ARCHITECTURE.md Phase 1-4). Establishes the file structure all subsequent work targets.
**Avoids:** Pitfall 1 (borrow conflicts -- design narrow signatures from the start), Pitfall 4 (overlay z-order -- enforce render order in orchestrator), Pitfall 8 (retro theme integrity -- layout computation accounts for theme-conditional areas).
**Build order within phase:** helpers.rs -> theme.rs -> controls -> progress -> idle -> artwork -> metadata -> lyrics -> chassis -> orchestrator.

### Phase 2: Spacing System and Proportional Layout
**Rationale:** Consistent spacing is the foundation that every visual improvement builds on (FEATURES.md dependency graph shows it enables artwork centering, metadata alignment, button distribution, and dead zone elimination). Must come before individual section polish.
**Delivers:** Spacing constants applied uniformly across all sections. `Layout::vertical/horizontal()` with `Constraint::Fill` replacing legacy patterns. `Layout::spacing()` for inter-element gaps. No dead zones at standard terminal sizes.
**Addresses:** Table stakes features: consistent spacing, no dead zones, proportional layout.
**Uses:** `Constraint::Fill(weight)`, `Layout::spacing()`, `Layout::vertical/horizontal()`, `Layout::areas::<N>()`.
**Avoids:** Pitfall 2 (Cassowary non-determinism -- Fill + Min strategy gives solver clear priorities), Pitfall 5 (cache thrashing -- static constraints, layout count under 16), Pitfall 7 (arithmetic panics -- minimum-area guards added).

### Phase 3: Section-Level Polish
**Rationale:** With spacing system in place and sections isolated, each section can be polished independently. Artwork centering, button distribution, metadata alignment, and lyrics highlight are all leaf operations that do not affect each other.
**Delivers:** Artwork vertically centered via `Rect::centered()`. Buttons evenly distributed via `Flex::SpaceBetween`. Metadata labels cleanly aligned. Lyrics highlight with accent color and graduated dimming.
**Addresses:** All remaining table stakes features plus P1 priorities from FEATURES.md.
**Uses:** `Rect::centered()`, `Rect::centered_vertically()`, `Flex::SpaceBetween`, style changes for lyrics.
**Avoids:** Pitfall 3 (ratatui-image re-encode -- snap artwork dimensions, debounce resize).

### Phase 4: Stretch Polish
**Rationale:** Differentiator features that enhance beyond "not broken." Only attempted after all table stakes are complete. These are lower risk because the foundation is solid.
**Delivers:** Adaptive artwork/info split with Min/Max constraints. Button label truncation at narrow widths. Separator line between columns. Lyrics edge padding.
**Addresses:** P2 features from FEATURES.md prioritization matrix.
**Uses:** `Constraint::Min/Max` for adaptive ratios, width-based label truncation logic.

### Phase Ordering Rationale

- **Phase 0 before Phase 1:** The Alignment migration is a mechanical, zero-risk commit that prevents type ambiguity during the refactoring. It should be its own commit so it does not contaminate the decomposition diff.
- **Phase 1 before Phase 2:** You cannot safely change layout constraints in a 455-line function. Decomposition first isolates each section so constraint changes have bounded blast radius.
- **Phase 2 before Phase 3:** The spacing system must exist before centering and distribution are applied, because centering and distribution depend on consistent padding values. FEATURES.md dependency graph confirms this: spacing enables artwork centering, metadata alignment, and button distribution.
- **Phase 3 before Phase 4:** Table stakes before differentiators. Ship "not broken" before "impressive."
- **Phase 3 items are parallelizable:** Artwork centering, button distribution, metadata alignment, and lyrics highlight are independent leaf operations that can be done in any order within Phase 3.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Decomposition):** Needs `/gsd:research-phase` -- borrow checker interactions with `StatefulProtocol` and `ThrobberState` are subtle. The exact function signatures for `draw_artwork()` need validation against ratatui-image's `render_stateful_widget` requirements.
- **Phase 3 (Artwork centering specifically):** Needs targeted research on ratatui-image dimension snapping and resize debouncing. The interaction between `Rect::centered()` output dimensions and `StatefulProtocol` cache invalidation is not fully documented.

Phases with standard patterns (skip research-phase):
- **Phase 0 (Prep):** Purely mechanical find-and-replace. No research needed.
- **Phase 2 (Spacing + Proportional):** Well-documented Ratatui 0.30 patterns. STACK.md provides exact migration code for every constraint replacement.
- **Phase 4 (Stretch):** Standard Ratatui constraint patterns. Min/Max usage is well-documented.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All APIs verified in local ratatui-core 0.1.0 source and docs.rs. No speculation -- every recommendation was confirmed to exist in the installed version. |
| Features | HIGH | Feature landscape drawn from 5 comparable Ratatui TUI players (rmpc, spotify-player, termusic, ncmpcpp, LyricsMPRIS-Rust). Table stakes vs. differentiators grounded in competitor analysis. |
| Architecture | HIGH | Decomposition pattern validated against Ratatui official docs (widget concepts, component architecture, best practices discussion). Build order accounts for borrow checker constraints verified against Rust semantics. |
| Pitfalls | HIGH | Every pitfall verified against at least two sources (Ratatui docs + codebase analysis, or community discussions + API documentation). ratatui-image re-encode warning comes from the library's own README. |

**Overall confidence:** HIGH

All four research files drew from official Ratatui documentation, verified API source code locally, and cross-referenced with real-world Ratatui applications. No finding relies on a single unverified source.

### Gaps to Address

- **ratatui-image dimension snapping:** The exact strategy for snapping artwork Rect dimensions to avoid re-encoding is not documented. The research identifies the problem and recommends "round to even numbers," but the optimal snap granularity (2 cells? 4 cells?) needs empirical testing during Phase 3.
- **Halfblock aspect ratio:** Halfblocks assume a 4:8 pixel ratio when font size is undetectable. The centering math for halfblock protocol may need different constants than Sixel/Kitty. This should be validated when implementing artwork centering.
- **Japanese CJK double-width labels:** The metadata alignment assumes consistent character widths, but CJK characters are double-width in terminal cells. The spacing system in Phase 2 needs to account for this when setting label column widths. The `unicode-width` crate (already a transitive dependency via ratatui) can help.
- **Layout cache size adequacy:** The default 16-entry LRU cache should be sufficient, but the total number of unique `Layout::split()` calls after decomposition should be counted and documented. If it exceeds 12, call `Layout::init_cache(32)` as a safety measure.

## Sources

### Primary (HIGH confidence)
- [Ratatui Layout Concepts](https://ratatui.rs/concepts/layout/) -- constraint solver, flex, spacing
- [Ratatui 0.30 Highlights](https://ratatui.rs/highlights/v030/) -- new APIs, Rect::centered(), alignment rename
- [Ratatui Rect API (docs.rs)](https://docs.rs/ratatui/latest/ratatui/layout/struct.Rect.html) -- centering methods, layout methods
- [Ratatui Constraint API (docs.rs)](https://docs.rs/ratatui/latest/ratatui/layout/enum.Constraint.html) -- Fill, priority order
- [Ratatui Widget Concepts](https://ratatui.rs/concepts/widgets/) -- Widget vs StatefulWidget patterns
- [Ratatui Component Architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/) -- decomposition patterns
- [ratatui-image README](https://github.com/ratatui/ratatui-image) -- StatefulProtocol re-encode warning
- [ratatui-macros API (docs.rs)](https://docs.rs/ratatui-macros/latest/ratatui_macros/) -- macro syntax
- Local source verification: ratatui-core 0.1.0 in cargo registry (rect.rs, layout.rs)

### Secondary (MEDIUM confidence)
- [Ratatui GitHub Discussion #220](https://github.com/ratatui/ratatui/discussions/220) -- MVC patterns, state management
- [Ratatui GitHub Discussion #164](https://github.com/ratatui/ratatui/discussions/164) -- ownership/borrowing challenges
- [Ratatui GitHub Discussion #592](https://github.com/ratatui/ratatui/discussions/592) -- Frame::render_widget ownership
- [Ratatui Breaking Changes](https://github.com/ratatui/ratatui/blob/main/BREAKING-CHANGES.md) -- HorizontalAlignment migration

### Competitor Analysis (MEDIUM confidence)
- [rmpc](https://github.com/mierak/rmpc) -- Ratatui 0.30 MPD client, configurable layout reference
- [spotify-player](https://github.com/aome510/spotify-player) -- Rust/Ratatui Spotify client, centering and layout reference
- [termusic](https://github.com/tramhao/termusic) -- Rust TUI music player
- [ncmpcpp](https://wiki.archlinux.org/title/Ncmpcpp) -- gold standard terminal music player
- [LyricsMPRIS-Rust](https://github.com/BEST8OY/LyricsMPRIS-Rust) -- lyrics highlight patterns

---
*Research completed: 2026-03-26*
*Ready for roadmap: yes*
