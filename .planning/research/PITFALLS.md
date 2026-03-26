# Pitfalls Research

**Domain:** Ratatui TUI layout refactoring (monolithic draw function decomposition and responsive layout)
**Researched:** 2026-03-26
**Confidence:** HIGH (verified against Ratatui 0.30 docs, codebase analysis, and community sources)

## Critical Pitfalls

### Pitfall 1: Borrow Conflict When Extracting `render_stateful_widget` Calls

**What goes wrong:**
Extracting parts of the monolithic `draw()` function into helper functions (e.g., `draw_artwork()`, `draw_controls()`) causes borrow checker errors when `render_stateful_widget` needs `&mut Frame` AND `&mut app.some_field` simultaneously. The current code passes `&mut App` to `draw()` and accesses `&mut app.throbber_state` and `&mut app.artwork_protocol` for stateful rendering (lines 751, 754 of `ui/mod.rs`). Extracting these into sub-functions that take `&mut Frame` and `&mut App` will compile fine, but further decomposition (e.g., a function that takes `&mut Frame` and only the fields it needs) runs into Rust's inability to do partial borrows through function signatures.

**Why it happens:**
Rust cannot express "borrow field A mutably but field B immutably" across function boundaries. The current monolithic function works because the compiler can see all field accesses in one scope and verify they don't conflict. The moment you extract to a function taking `&mut App`, the compiler must assume the whole struct is mutably borrowed.

**How to avoid:**
1. Pass individual fields to sub-functions, not `&mut App`. For artwork rendering: `draw_artwork(f, art_rect, &mut app.throbber_state, &mut app.artwork_protocol, app.is_loading_artwork, &app.config, theme)`.
2. Alternatively, pre-compute layout `Rect` values in a pure `fn layout(area: Rect, config: &Config) -> LayoutRects` step that needs no mutable access, then pass rects + mutable fields to render functions separately.
3. For the `StatefulImage` and `Throbber` specifically: extract the mutable state references before the sub-function call using temporary variables.

**Warning signs:**
- Compiler error E0499 (cannot borrow `*app` as mutable more than once) or E0502 (cannot borrow `*app` as immutable because it is also borrowed as mutable) when splitting functions.
- Needing to clone data just to satisfy the borrow checker -- this indicates the decomposition boundary is wrong.

**Phase to address:**
Phase 1 (function extraction) -- this is the very first thing that will go wrong. Design the function signatures before writing code.

---

### Pitfall 2: Cassowary Constraint Solver Non-Determinism at Small Terminal Sizes

**What goes wrong:**
Ratatui uses the Cassowary constraint solver, and when constraints cannot all be satisfied (common at small terminal sizes), the solver returns an **arbitrary, non-deterministic** result. The current code uses `Constraint::Min(10)` for the display area, `Constraint::Length(3)` for tuner, and `Constraint::Length(3)` for controls. At terminal heights under ~16 rows, the `Min(10)` fights with the two `Length(3)` constraints and results become unpredictable -- the controls might disappear, or the display area might get zero height.

The code also does `display_area.width > 50` (line 699) to decide whether to show artwork, but never guards against `display_area.height` being zero or near-zero, which would cause the artwork centering math (`artwork_column.height.saturating_sub(char_height) / 2`) to produce a zero-height render area.

**Why it happens:**
The Cassowary algorithm tries to satisfy constraints as closely as possible, but the "close" solution is implementation-defined. Mixing `Percentage`, `Min`, `Length`, and `Ratio` constraints in the same layout compounds the problem. The official docs warn: "The specific result is non-deterministic when this occurs."

**How to avoid:**
1. Use `Constraint::Min` for elements that must appear (controls, progress bar) and `Constraint::Fill(1)` for the flexible content area. This gives the solver a clear priority.
2. Guard every render call with a minimum-area check: `if area.height >= 3 && area.width >= 10 { draw_controls(f, area, ...); }`. The lyrics already do this (`lyrics_area.height > 2`), but other sections do not.
3. Test with `TestBackend::new(40, 10)` and `TestBackend::new(80, 8)` to catch zero-area crashes during development.
4. Avoid mixing `Percentage` with `Length`/`Min` in the same layout split. The horizontal content split (lines 701-715) uses `Percentage(42)` + `Length(1)` + `Percentage(57)` -- these percentages sum to 99% leaving a remainder column, and the `Length(1)` separator interacts unpredictably with them at narrow widths.

**Warning signs:**
- Widgets appearing at (0,0) or overlapping other widgets at small sizes.
- Controls or progress bar disappearing when the terminal is resized.
- Panic in index access (e.g., `btn_layout[i]` on line 1063 already has a guard, but similar patterns elsewhere may not).

**Phase to address:**
Phase 1 (layout restructuring) -- define the constraint strategy once, verify at minimum sizes before building on top.

---

### Pitfall 3: ratatui-image StatefulProtocol Blocking Re-encode on Layout Change

**What goes wrong:**
`StatefulImage` re-encodes the image at render time when the render area changes size. Encoding Sixel or Kitty protocol data for a ~600x600 image is CPU-intensive. If the layout refactoring changes the artwork `Rect` dimensions on every frame (even by 1 cell), the image will re-encode every frame, causing visible lag and dropped frames in the 50ms poll loop. This is especially problematic during terminal resize events, where the area changes continuously.

The current code computes artwork size from `artwork_column.width` (line 722), so any change to the column width (e.g., switching from `Constraint::Percentage(42)` to a `Constraint::Fill`-based proportional system) will produce different pixel values and trigger re-encoding.

**Why it happens:**
ratatui-image's `StatefulProtocol` caches the last encoded image for a specific area size. If the area changes, it re-encodes. The ratatui-image README explicitly warns: "The resizing and encoding is blocking, and since it happens at render-time it is a good idea to offload that to another thread or async task, if the UI must be responsive."

**How to avoid:**
1. Snap the artwork Rect dimensions to stable values (e.g., round to even numbers or to the nearest 2-cell increment) so small layout adjustments don't trigger re-encoding.
2. During terminal resize events, show a placeholder (the existing throbber/loader) and debounce the artwork re-render by 200-300ms.
3. Keep the artwork column constraint as a `Constraint::Percentage` or `Constraint::Ratio` -- these produce stable values at any given terminal size. Avoid `Constraint::Fill` for the artwork column because it distributes remaining pixels differently depending on other constraints.
4. Consider offloading the encode to the existing Tokio runtime (the app already spawns `artwork_task` for downloads -- extend this pattern to re-encoding).

**Warning signs:**
- CPU usage spikes during terminal resize.
- Artwork flickering or briefly showing blank/throbber during layout adjustments.
- Frame rendering time exceeding the 50ms poll interval (visible as input lag).

**Phase to address:**
Phase 2 (artwork centering) -- this is when artwork Rect dimensions will change. The debounce strategy should be designed before touching artwork layout.

---

### Pitfall 4: Breaking the Settings Overlay Z-Order During Decomposition

**What goes wrong:**
The settings overlay (`settings_menu.render()`) is rendered last in `draw()` (line 1068-1070), after all other widgets. It uses `Clear` to blank its area and then draws on top. If the draw function is decomposed and the settings render is called before other widgets (or in the wrong order), the overlay will be drawn under the main UI, becoming invisible. Similarly, if any extracted sub-function renders to the full `f.area()` (as the background fill on line 622 does), it will overwrite the overlay.

**Why it happens:**
Ratatui uses a buffer-based immediate-mode rendering model. Widgets drawn later overwrite widgets drawn earlier in the same area. There is no z-index system. Render order IS z-order. The settings overlay depends on being absolutely last.

**How to avoid:**
1. Make render order explicit in the decomposed `draw()` function. The orchestrator function should call sub-functions in strict visual layer order: background -> chassis -> content -> progress -> controls -> overlays.
2. Never allow an extracted sub-function to render to an area that encompasses the overlay region without first checking `app.settings_menu.is_open`.
3. Add a comment block at the top of the orchestrator `draw()` documenting the render order contract.

**Warning signs:**
- Settings overlay not appearing after refactoring.
- Settings overlay appearing but with visual corruption (partially overwritten).
- Flickering when opening/closing settings.

**Phase to address:**
Phase 1 (function extraction) -- the render order contract must be established before extracting any sub-functions.

---

### Pitfall 5: Layout Cache Thrashing from Dynamic Constraint Construction

**What goes wrong:**
Ratatui caches `Layout::split()` results in a thread-local LRU cache (default 16 entries). The cache key is the combination of layout parameters and the input `Rect`. The current code constructs new `Layout` objects on every frame (lines 673-680, 701-715, 728-744, etc.). If the refactoring introduces additional dynamic layouts (e.g., calculating constraints based on `app.animation_frame`, content length, or theme), each unique constraint set creates a new cache entry. With a 16-entry LRU cache and multiple dynamic layouts, the cache will thrash on every frame, re-running the Cassowary solver.

The control button layout is already dynamic: `Constraint::Length(btn_width)` where `btn_width = control_area.width / controls.len() as u16` (line 1028). This is stable for a given terminal width, but adding more dynamic calculations multiplies the problem.

**Why it happens:**
The LRU cache works on exact parameter equality. A layout with `[Percentage(42), Length(1), Percentage(57)]` and one with `[Percentage(43), Length(1), Percentage(56)]` are two separate cache entries. The default cache size of 16 entries sounds generous but is shared across ALL layouts in the application.

**How to avoid:**
1. Keep constraints static where possible. Use the same constraint values every frame; let Flex and container sizing handle proportional distribution.
2. If constraints must be dynamic, call `Layout::init_cache(64)` at startup to increase the cache size.
3. Pre-compute all layout constraints at the start of `draw()` based on terminal size, not based on content. Terminal size changes far less frequently than content.
4. Avoid constructing `Layout` objects inside loops (the current button layout in a `for` loop is fine since it's constructed once, but refactoring might introduce nested layout splits per-item).

**Warning signs:**
- UI feeling sluggish despite no I/O changes.
- Profiling showing time spent in `cassowary` solver on every frame.
- More than 16 distinct `Layout::split()` calls in a single `draw()` invocation.

**Phase to address:**
Phase 1 (layout restructuring) -- establish the constraint strategy as static layouts up front. Count total split calls in the refactored code.

---

### Pitfall 6: Ratatui 0.30 `Alignment` to `HorizontalAlignment` Migration Landmine

**What goes wrong:**
Ratatui 0.30 renamed `Alignment` to `HorizontalAlignment` and added a `VerticalAlignment` enum. A type alias preserves backwards compatibility for most uses, but glob imports (`use Alignment::*`) and pattern matching on the enum will break. The current codebase uses `Alignment::Center` in 8 places (lines 579, 611, 639, 652, 759, 959, 1060 of `ui/mod.rs`) and imports it as `layout::Alignment`. During the refactoring, if `use ratatui::prelude::*` is introduced (a common pattern in Ratatui examples), the glob import may pull in `HorizontalAlignment` and `Alignment` simultaneously, causing ambiguity errors.

Additionally, the new `Rect::centered()` method and `VerticalAlignment` enum are now available and should be used for the artwork centering work instead of the manual vertical centering math on lines 725-735.

**Why it happens:**
The type alias works for direct `Alignment::Center` usage, but refactoring naturally leads to changing imports, and the Ratatui 0.30 examples use `HorizontalAlignment`. Copy-pasting from current docs while keeping the old import creates ambiguity.

**How to avoid:**
1. At the start of the refactoring, migrate all `Alignment` references to `HorizontalAlignment` in a standalone commit. This is mechanical and safe.
2. Use the new `Rect::centered()`, `Rect::centered_vertically()`, and `Rect::centered_horizontally()` helpers for centering logic instead of manual `Layout` + padding arithmetic.
3. Do not mix `use ratatui::prelude::*` with explicit layout imports.

**Warning signs:**
- Compiler warnings about deprecated type aliases.
- Ambiguous type errors mentioning both `Alignment` and `HorizontalAlignment`.
- New code using `Alignment` while examples show `HorizontalAlignment`.

**Phase to address:**
Phase 0 (pre-refactoring housekeeping) -- do this before any layout work as a mechanical, low-risk preparatory step.

---

### Pitfall 7: Arithmetic Panics from Hardcoded Layout Constants

**What goes wrong:**
The current code has several hardcoded dimension assumptions that produce panics or garbage at unexpected terminal sizes:
- `meta_height` is set to 7 or 10 (line 787) depending on column layout, but is never checked against available height.
- `h_padding = 2` for artwork (line 721) and `artwork_column.width.saturating_sub(h_padding * 2)` -- safe, but `side / 2` for character height (line 725) produces 0 when `side < 2`, making the artwork area invisible.
- `scroll_text()` width calculation: `metadata_area.width.saturating_sub(6) as usize` (line 910) -- returns 0 at widths under 6, causing `scroll_text` to return empty strings.
- `btn_width = control_area.width / controls.len() as u16` (line 1028) -- integer division truncation means some horizontal space is always unused. With 7 controls, `80 / 7 = 11` per button, using 77 of 80 columns.

**Why it happens:**
Layout code is written for the developer's terminal size and tested visually. Edge cases at small or unusual sizes (e.g., 40x10, 200x50, 80x8) are never exercised. The `saturating_sub` usage prevents panics in some places but produces invisible zero-area widgets.

**How to avoid:**
1. Replace hardcoded padding constants with proportional calculations or constants gated on available area: `let h_padding = if area.width > 60 { 2 } else { 1 };`
2. Add minimum-area guards before every render call. If the area is too small to meaningfully render, skip it or show a compact fallback.
3. For button distribution, use `Constraint::Fill(1)` for each button instead of `Constraint::Length(btn_width)`. This distributes space evenly with no remainder. This directly addresses the project requirement "Control buttons evenly distributed across terminal width at any size."
4. Write `TestBackend` tests at 3-4 different sizes: small (40x10), medium (80x24), large (120x40), ultra-wide (200x20).

**Warning signs:**
- Widgets appearing as blank/empty at small terminal sizes.
- Uneven gaps between buttons or between sections.
- Layout "jumping" when terminal is resized by a single column.

**Phase to address:**
Phase 2 (proportional spacing and distribution) -- this is where the button layout fix and spacing constants should be addressed.

---

### Pitfall 8: Losing Retro Theme Visual Integrity During Layout Abstraction

**What goes wrong:**
The current `draw()` function has `if theme.is_retro { ... }` branches in 7+ places that add chassis borders, double-border screens, scanline effects, thick borders on buttons, and dark backgrounds. When decomposing into sub-functions, it's tempting to make each sub-function theme-agnostic and pass theme as a parameter. But the retro theme's visual integrity depends on the RELATIONSHIP between elements: the chassis border must touch the screen border, the scanline effect must not bleed into the progress bar, the button borders must align with the chassis. If sub-functions compute their own local padding and borders, the retro theme's pixel-precise alignment breaks.

**Why it happens:**
Theme-conditional rendering is interleaved with layout logic. The chassis `inner` area (line 654-668) is computed before the main layout split and affects all downstream areas. Extracting `draw_chassis()` as an independent function means its `inner` area must be returned and used by all subsequent sub-functions -- but the clean theme (non-retro) has no chassis, so the function interface becomes awkward.

**How to avoid:**
1. Separate layout computation from rendering completely. Compute a `LayoutResult` struct that contains all `Rect`s, then pass these rects to render functions. The chassis/no-chassis distinction affects layout computation, not rendering.
2. The layout phase should return: `chassis_inner` (= `area` for clean theme, = `chassis_block.inner(area)` for retro), `screen_inner`, `tuner_area`, `control_area`, `artwork_area`, `metadata_area`, `lyrics_area`.
3. Test both retro and clean themes at every layout test size. The existing test only uses the default theme.

**Warning signs:**
- Retro chassis border not touching the screen border (1-pixel gap).
- Scanline effect extending into the progress bar or controls.
- Clean theme having unexpected borders or padding.

**Phase to address:**
Phase 1 (function extraction) -- the layout computation struct must account for theme-conditional areas from the start.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Passing `&mut App` to every sub-function instead of individual fields | Quick extraction, no signature design needed | Borrow conflicts block further decomposition; functions have access to unrelated state | Never -- invest in proper signatures from the start |
| Using `Constraint::Percentage` for artwork column with remainders summing to 99% | Looks proportional | The missing 1% creates a gap or non-deterministic column assignment at certain terminal widths | Use `Constraint::Ratio(42, 100)` or `Constraint::Fill` instead |
| Hardcoding `meta_height = 7` / `meta_height = 10` | Works at common terminal sizes | Wastes space at large sizes, overflows at small sizes | Only during prototyping; replace with proportional before merge |
| Skipping `TestBackend` tests for extracted sub-functions | Faster initial refactoring | Layout regressions go unnoticed until manual testing | Never -- layout tests are cheap and the existing test infrastructure already exists |
| Computing layout inside render functions (coupling layout + draw) | Fewer function parameters | Cannot pre-compute layout, cannot test layout independently, cache thrashing | Only for leaf widgets that need exactly one local layout split |

## Integration Gotchas

Common mistakes when working with ratatui-image and terminal rendering.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| ratatui-image `StatefulProtocol` | Changing the render Rect every frame causes blocking re-encode | Snap dimensions to stable values; debounce during resize |
| ratatui-image with halfblocks | Assuming halfblocks have the same aspect ratio as Sixel/Kitty | Halfblocks assume 4:8 pixel ratio when font size is undetectable; artwork centering math must account for this |
| Crossterm resize events | Treating `Event::Resize` the same as a regular frame | Resize events fire rapidly during drag; layout should debounce or use the final size |
| `Throbber` stateful widget | Forgetting to tick `throbber_state` when the artwork is loading | The throbber only animates if `app.throbber_state.calc_next()` is called on the timer; moving the throbber to a sub-function doesn't change this, but it's easy to lose track of |
| Settings overlay with `Clear` widget | Rendering `Clear` AFTER the settings content instead of BEFORE | `Clear` must come first to blank the area, then draw content on top. Reversing order makes content invisible. |

## Performance Traps

Patterns that work at normal scale but degrade at high frame rates or large terminals.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Re-running Cassowary solver 10+ times per frame | UI latency exceeds 50ms poll interval, input feels laggy | Keep Layout splits under 16 unique configurations per frame; use `Layout::init_cache()` if needed | At 15+ unique Layout::split calls per draw cycle |
| Constructing new `Layout` objects with dynamic constraints every frame | Cache misses on every frame, solver runs every time | Use static constraint arrays; only the input Rect should change | When any constraint value changes frame-to-frame |
| Image re-encoding on every render when Rect dimensions change by 1 cell | CPU spike, artwork flicker, frame drops | Snap artwork dimensions to even numbers; debounce resize | Any terminal resize or layout proportion change |
| `scroll_text()` running on every metadata field every frame | String allocation per field per frame | Current implementation is already optimized with `Cow`, but extracting it into a sub-function that clones the result would regress | If refactoring introduces unnecessary cloning of the Cow return |
| Deep nesting of `Layout::split` calls in extracted sub-functions | Each nested split is a separate cache entry; 4 levels deep with 3 splits each = 12 cache entries | Flatten layout computation; compute all rects in one pass at the top of draw() | When total split count exceeds cache size (16) |

## UX Pitfalls

Common user experience mistakes in TUI layout refactoring.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Artwork area changing size during track changes | Distracting visual jump; whole layout shifts | Keep artwork column width stable; only change the inner artwork rect |
| Progress bar height changing when layout reflows | User loses sense of playback position | Fix progress bar at `Length(3)` unconditionally; never make it flexible |
| Lyrics scroll position resetting on layout reflow | User loses their place in the lyrics | Preserve `current_index` and re-compute scroll offset from it after layout change |
| Buttons truncating labels at narrow widths instead of hiding the shortcut | User sees "PLA [S" instead of "PLAY" | Truncate the key hint `[SPC]` first, keep the label; show just the label below threshold |
| Dead space between artwork and text columns | Looks broken/unfinished, especially in clean theme | Use `Constraint::Fill` or explicit separator widget; never leave unaccounted space |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Artwork centering:** Looks centered at 80x24 but off-center at 120x40 -- verify centering at 3+ terminal sizes
- [ ] **Button distribution:** Looks even at 80 columns but has 3-pixel gap at the right at 81 columns -- verify with `Constraint::Fill(1)` that remainder distributes, not accumulates
- [ ] **Retro theme chassis:** Looks fine in clean theme but chassis border detached from screen border after refactoring -- test both retro and clean themes
- [ ] **Lyrics area:** Fills space at 80x24 but only uses top half at 80x48 -- verify lyrics expand to fill available vertical space
- [ ] **Japanese text labels:** Widths are correct for ASCII labels but CJK characters are double-width, causing alignment issues -- verify with `is_jp = true` at all test sizes
- [ ] **Small terminal:** Looks great at 80x24 but panics at 40x10 -- add minimum-size tests
- [ ] **Settings overlay position:** Centered at 80x24 but off-screen at 40x10 -- verify `popup_width = 60.min(area.width - 4)` still works after layout changes

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Borrow checker conflicts from `&mut App` signatures | MEDIUM | Redesign function signatures to take individual fields; mechanical but touches every sub-function |
| Cassowary non-determinism at small sizes | LOW | Add `Min` guards and minimum-area checks to affected render calls; localized fixes |
| ratatui-image re-encode lag | MEDIUM | Add dimension snapping + debounce; requires adding resize event handling to the event loop |
| Settings overlay Z-order broken | LOW | Move settings render to guaranteed-last position; 1-line fix once diagnosed |
| Layout cache thrashing | LOW | Call `Layout::init_cache(64)` and/or reduce unique layout configurations; 1-2 line fix |
| Retro theme visual breakage | HIGH | Requires re-examining the relationship between all decomposed functions and theme areas; may need to restructure the layout computation |
| Hardcoded constant overflow at small sizes | MEDIUM | Audit all hardcoded values and add proportional fallbacks; grep for raw integer literals in layout code |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Borrow conflict on extraction | Phase 1: Function extraction | Code compiles with sub-functions taking individual fields, not `&mut App` |
| Cassowary non-determinism | Phase 1: Layout restructuring | `TestBackend` tests pass at 40x10, 80x24, and 120x40 |
| ratatui-image re-encode lag | Phase 2: Artwork centering | No artwork flicker during terminal resize; dimensions snap to stable values |
| Settings overlay Z-order | Phase 1: Function extraction | Settings overlay visible and correct in both themes after extraction |
| Layout cache thrashing | Phase 1: Layout restructuring | Total unique `Layout::split` calls per frame counted and documented; under 16 |
| `Alignment` migration | Phase 0: Prep commit | All `Alignment` references updated to `HorizontalAlignment`; new `Rect::centered()` helpers used |
| Arithmetic at small sizes | Phase 2: Proportional spacing | `TestBackend` tests at 40x10 render without panics or invisible widgets |
| Retro theme integrity | Phase 1: Function extraction | Visual verification of retro theme at 3 sizes; chassis-to-screen border alignment preserved |

## Sources

- [Ratatui Layout Concepts](https://ratatui.rs/concepts/layout/) -- Cassowary non-determinism warning, excess space anti-pattern
- [Ratatui v0.30.0 Highlights](https://ratatui.rs/highlights/v030/) -- `Alignment` rename, `Rect::centered()` helpers, Flex changes
- [Ratatui Breaking Changes](https://github.com/ratatui/ratatui/blob/main/BREAKING-CHANGES.md) -- `HorizontalAlignment` migration, layout cache feature flag
- [Ratatui Centering Recipe](https://ratatui.rs/recipes/layout/center-a-widget/) -- `Rect::centered()` usage pattern
- [Ratatui Widget Concepts](https://ratatui.rs/concepts/widgets/) -- `Widget for &MyWidget` vs `StatefulWidget` patterns, borrow implications
- [Ratatui GitHub Discussion #164](https://github.com/ratatui/ratatui/discussions/164) -- State management and borrowing challenges
- [Ratatui GitHub Discussion #592](https://github.com/ratatui/ratatui/discussions/592) -- Ownership in `Frame::render_widget`
- [ratatui-image README](https://github.com/ratatui/ratatui-image) -- Blocking re-encode warning, protocol differences
- [Ratatui Snapshot Testing Recipe](https://ratatui.rs/recipes/testing/snapshots/) -- TestBackend + insta for layout regression
- [Ratatui Constraint Explorer](https://ratatui.rs/examples/layout/constraint-explorer/) -- Interactive constraint behavior reference
- Codebase analysis: `src/ui/mod.rs` lines 617-1071, `src/ui/settings.rs`, `Cargo.toml`

---
*Pitfalls research for: Ratatui TUI layout refactoring*
*Researched: 2026-03-26*
