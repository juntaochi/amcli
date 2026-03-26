# Phase 2: Proportional Layout & Spacing System - Research

**Researched:** 2026-03-26
**Domain:** Ratatui 0.30 constraint solver -- Fill weights, Layout::spacing(), proportional distribution
**Confidence:** HIGH

## Summary

This phase replaces legacy `Constraint::Percentage` splits with `Constraint::Fill` weighted proportional layout and establishes a unified spacing constant system. The current orchestrator in `draw()` uses `Percentage(42)/Length(1)/Percentage(57)` for the artwork/info split, which sums to 99% plus a 1-cell separator, leaving visible gaps at certain terminal widths. `Constraint::Fill(3)/Fill(4)` gives an exact 3:4 ratio with no remainder. The `Layout::spacing(1)` API replaces the manual `Constraint::Length(1)` separator entirely.

All required APIs are available in the project's existing ratatui 0.30.0 dependency. No new crate dependencies are needed. The changes are confined to `src/ui/mod.rs` -- the orchestrator function `draw()` (lines 1001-1100) and the imports at line 4.

**Primary recommendation:** Define 3 spacing constants at the top of `src/ui/mod.rs`, replace all `Layout::default().direction(...)` calls with `Layout::vertical()`/`Layout::horizontal()`, replace `Percentage` with `Fill` for proportional areas, add `Min` guards for narrow terminals, and apply `Layout::spacing()` for consistent inter-section gaps.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- 3 spacing levels as constants at top of src/ui/mod.rs: TIGHT(0), NORMAL(1), SECTION(2)
- Apply spacing between artwork/info columns, between metadata/lyrics vertically, around progress bar and controls
- Constants defined as `const` block co-located with rendering code
- Replace Constraint::Percentage(42)/Percentage(57) with Constraint::Fill(3)/Constraint::Fill(4) for artwork/info split
- Artwork column: Min(20) constraint to protect at narrow widths
- Info column: Min(30) constraint to protect at narrow widths
- Progress bar: Keep Length(3) -- fixed content height
- Controls area: Keep Length(3) -- fixed button row height
- Artwork visibility threshold: Keep current `width > 50`
- Artwork/info ratio at wide terminals: 40/60 split (slightly favor info for lyrics readability)
- Two-column metadata threshold: Keep current logic

### Claude's Discretion
- Exact implementation of Layout::spacing() integration
- Whether to use Flex variants for any sub-layouts
- How to handle the metadata/lyrics vertical split proportionally

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LAYT-03 | Layout uses proportional Fill constraints instead of percentage-based splits that leave gaps | Fill(3)/Fill(4) solver behavior verified in ratatui-core source; test cases confirm exact proportional distribution with no remainder |
| LAYT-04 | Artwork/info split ratio adapts to terminal width using Min/Max constraints | Min constraints have STRONG*100 priority, always honored before Fill (MEDIUM priority); Min(20)+Fill(3) / Min(30)+Fill(4) pattern verified in solver |
| VISL-01 | Consistent spacing system with unified constants replacing ad-hoc padding values | Layout::spacing() API verified; Spacing enum accepts u16 directly; 7 ad-hoc spacing locations identified for replacement |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Build/verify: `make verify` (fmt check + clippy + test + build)
- Commit format: `<type>(<scope>): <subject>` (e.g., `feat(ui): add spacing constants`)
- Never block the UI draw loop with I/O
- Use `anyhow::Result` for application logic
- macOS only; requires Apple Music app

## Standard Stack

### Core (Already Installed -- No Changes)

| Library | Version | Purpose | Verified |
|---------|---------|---------|----------|
| ratatui | 0.30.0 | TUI framework -- `Layout`, `Constraint::Fill`, `Spacing`, `Flex` | In Cargo.toml |
| ratatui-core | 0.1.0 | Underlying layout solver (transitive dep) | In Cargo.lock |

### APIs Used in This Phase

| API | Import Path | Purpose |
|-----|-------------|---------|
| `Layout::vertical()` | `ratatui::layout::Layout` | Replaces `Layout::default().direction(Direction::Vertical)` |
| `Layout::horizontal()` | `ratatui::layout::Layout` | Replaces `Layout::default().direction(Direction::Horizontal)` |
| `Layout::areas::<N>()` | `ratatui::layout::Layout` | Compile-time destructuring -- replaces `split()[index]` |
| `Layout::spacing()` | `ratatui::layout::Layout` | Uniform gap between layout segments |
| `Constraint::Fill(u16)` | `ratatui::layout::Constraint` | Weighted proportional space distribution |
| `Constraint::Min(u16)` | `ratatui::layout::Constraint` | Already imported; used for narrow-width protection |

### Import Changes Required

Current (line 4):
```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    ...
};
```

After (no longer needs `Direction`; does not need `Flex` or `Spacing` since they are passed as method args, not used as standalone types):
```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    ...
};
```

Note: `Direction` import can remain for backward compat with any other uses. `Layout::spacing()` accepts `u16` directly via `Into<Spacing>`, so no explicit `Spacing` import is needed. No `Flex` import needed for this phase (no Flex usage in scope).

**Confidence:** HIGH -- verified from ratatui-core 0.1.0 source at `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/`.

## Architecture Patterns

### Spacing Constants (New)

Place immediately after the existing theme constants (after line 33) in `src/ui/mod.rs`:

```rust
// Spacing system: unified constants for layout gaps
const SPACING_TIGHT: u16 = 0;   // No gap -- adjacent elements touching
const SPACING_NORMAL: u16 = 1;  // 1-cell gap -- between sibling sections
const SPACING_SECTION: u16 = 2; // 2-cell gap -- between major sections
```

These are `u16` because `Layout::spacing()` accepts `impl Into<Spacing>`, and `u16` converts directly to `Spacing::Space(n)`.

### Orchestrator Layout Pattern (Replacement)

**Current** (lines 1009-1017):
```rust
let main = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Min(10),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(chassis_inner);
let (display_area, tuner_area, control_area) = (main[0], main[1], main[2]);
```

**Replace with:**
```rust
let [display_area, tuner_area, control_area] = Layout::vertical([
    Constraint::Fill(1),
    Constraint::Length(3),
    Constraint::Length(3),
])
.areas(chassis_inner);
```

Key changes:
- `Layout::vertical()` replaces `Layout::default().direction(Direction::Vertical)`
- `Fill(1)` replaces `Min(10)` -- idiomatic "take all remaining space"
- `.areas()` replaces `.split()` + indexing -- compile-time checked destructuring
- No spacing here because tuner/control bars are visually adjacent (bordered blocks)

### Artwork/Info Split Pattern (Replacement)

**Current** (lines 1021-1033):
```rust
let artwork_constraints: &[Constraint] = if show_artwork {
    &[
        Constraint::Percentage(42),
        Constraint::Length(1),
        Constraint::Percentage(57),
    ]
} else {
    &[Constraint::Percentage(100)]
};
let content_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(artwork_constraints)
    .split(screen_inner);
```

**Replace with:**
```rust
if show_artwork {
    let [artwork_col, info_col] = Layout::horizontal([
        Constraint::Fill(3),
        Constraint::Fill(4),
    ])
    .spacing(SPACING_NORMAL)
    .areas(screen_inner);
    // use artwork_col, info_col
} else {
    let info_col = screen_inner;
    // use info_col (full width)
}
```

Key changes:
- `Fill(3)/Fill(4)` gives 3:4 ratio (~43%/57%) with no remainder gap
- `.spacing(SPACING_NORMAL)` adds a 1-cell gap *managed by the layout engine* -- replaces the manual `Constraint::Length(1)` separator column
- Eliminates the 3-element constraint array and `content_layout[2]` indexing
- The `else` branch becomes a simple assignment, no layout computation needed

### Metadata/Lyrics Split Pattern (Replacement)

**Current** (lines 1057-1071) has three branches:
```rust
// Branch 1: no artwork + has lyrics → horizontal 45/55 split
// Branch 2: has lyrics + enough height → vertical Length(meta)/Min(0)
// Branch 3: no lyrics → full area for metadata
```

**Replace with:**
```rust
let (metadata_area, lyrics_area) = if !show_artwork && has_lyrics {
    let [meta, lyrics] = Layout::horizontal([
        Constraint::Fill(2),
        Constraint::Fill(3),
    ])
    .spacing(SPACING_NORMAL)
    .areas(info_chunk);
    (meta, lyrics)
} else if has_lyrics && info_height > meta_height + 2 {
    let [meta, lyrics] = Layout::vertical([
        Constraint::Length(meta_height as u16),
        Constraint::Fill(1),
    ])
    .spacing(SPACING_NORMAL)
    .areas(info_chunk);
    (meta, lyrics)
} else {
    (info_chunk, Rect::default())
};
```

Key changes:
- `Percentage(45)/Percentage(55)` becomes `Fill(2)/Fill(3)` (same ~40/60 ratio, no rounding gap)
- `Min(0)` for lyrics becomes `Fill(1)` (idiomatic "take remaining space")
- `.spacing(SPACING_NORMAL)` adds consistent gap between metadata and lyrics areas

### Min Guard Pattern for Narrow Terminals

The CONTEXT.md specifies Min(20) for artwork and Min(30) for info. Ratatui's constraint solver does not support compound constraints on a single segment (you cannot say "Fill(3) AND Min(20)" on one constraint). Instead, use a **conditional constraint pattern**:

```rust
if show_artwork {
    let available = screen_inner.width;
    // If terminal too narrow for both minimums + spacing, give full width to info
    let artwork_constraints = if available >= 20 + 30 + SPACING_NORMAL {
        [Constraint::Fill(3), Constraint::Fill(4)]
    } else {
        // Fallback: artwork gets minimum, info gets rest
        [Constraint::Min(20), Constraint::Fill(1)]
    };
    let [artwork_col, info_col] = Layout::horizontal(artwork_constraints)
        .spacing(SPACING_NORMAL)
        .areas(screen_inner);
    // ...
}
```

**Why this works:** The existing `width > 50` threshold already prevents artwork from showing at very narrow widths. The Min guards provide a secondary defense for the 50-80 column range where both columns exist but one could be squeezed to uselessness.

**Alternative approach (simpler, recommended):** Since the `width > 50` threshold already guards against narrow terminals, the Fill(3)/Fill(4) split alone is sufficient for most cases. At width=51 (the minimum where artwork shows), Fill(3)/Fill(4) gives ~22/29 columns, which is close to the Min(20)/Min(30) guards. The Min guards add marginal value. Recommend implementing the simple Fill(3)/Fill(4) approach first, then adding Min guards only if testing reveals edge cases.

**Confidence:** HIGH -- `configure_fill_constraints()` in ratatui-core source confirms Fill weight behavior. Test case `rand_fill4` confirms `[Fill(1), Fill(3), Min(50), Fill(2), Fill(4)]` resolves correctly with Min honored first.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Inter-section spacing | Manual `Constraint::Length(1)` separator columns | `Layout::spacing(SPACING_NORMAL)` | Layout engine manages spacers automatically; eliminates phantom `content_layout[1]` index |
| Proportional splits | `Constraint::Percentage(42)` + manual gap compensation | `Constraint::Fill(3)/Fill(4)` | Exact proportional distribution, no 99%-sum rounding error |
| Array destructuring from layout | `.split(area)` + `main[0]`, `main[1]`, `main[2]` indexing | `.areas::<N>(area)` | Compile-time checked; panics at compile time if N mismatches |
| Vertical fill remaining space | `Constraint::Min(10)` as "take whatever is left" | `Constraint::Fill(1)` | `Min(10)` is a side-effect hack; `Fill(1)` communicates intent |
| Layout direction setup | `Layout::default().direction(Direction::Vertical)` | `Layout::vertical([...])` | One call instead of three chained calls |

## Common Pitfalls

### Pitfall 1: Spacing Does Not Apply with SpaceBetween/SpaceEvenly/SpaceAround
**What goes wrong:** Adding `.spacing(1)` to a layout that also uses `.flex(Flex::SpaceBetween)` -- the spacing is silently ignored.
**Why it happens:** The Flex variants that distribute space between segments handle their own spacer sizing. The `spacing()` value is overridden by the flex spacer algorithm.
**How to avoid:** For this phase, do NOT combine `.spacing()` with `.flex()`. The phase only needs `.spacing()` on standard `Layout::horizontal`/`Layout::vertical` with `Flex::Start` (the default).
**Warning signs:** Spacing looks wrong despite setting the value -- check if a `.flex()` call is present.
**Source:** ratatui-core layout.rs line 498-499: "spacing will not be applied for SpaceAround, SpaceEvenly and SpaceBetween"

### Pitfall 2: Fill(0) Does Not Mean "Zero Width"
**What goes wrong:** Using `Fill(0)` expecting it to collapse a segment. Instead, `Fill(0)` is treated as "equal weight to other Fill(0) segments" and shares remaining space equally.
**Why it happens:** The solver treats 0-weight fills specially -- they all get equal share of remaining space after non-zero-weight fills are satisfied.
**How to avoid:** Never use `Fill(0)`. Use `Length(0)` if you need a zero-width segment, or omit the segment entirely.
**Source:** ratatui-core test cases `zero_fill1` through `zero_fill6` at line 2314: `[Fill(0), Fill(1), Fill(0)]` produces `[0..0, 0..100, 100..100]` only because Fill(1) takes priority.

### Pitfall 3: Spacing Eats Available Space Before Fill Distributes
**What goes wrong:** Setting `spacing(2)` on a layout with 3 segments adds 2*2=4 cells of spacing. If the available area is narrow (say 50 cells), Fill weights distribute only 46 cells, not 50.
**Why it happens:** Spacing is deducted from available area before the constraint solver runs.
**How to avoid:** Account for spacing when reasoning about minimum viable widths. With `SPACING_NORMAL(1)` and 2 Fill segments, only 1 cell is consumed by spacing. This is negligible.
**Warning signs:** Elements appear slightly smaller than expected at narrow terminal widths.

### Pitfall 4: areas::<N>() Panics If N Mismatches Constraint Count
**What goes wrong:** `Layout::vertical([Fill(1), Length(3)]).areas::<3>(area)` panics at runtime, not compile time.
**Why it happens:** The N in `areas::<N>()` is a const generic checked at runtime via `try_into()`. The compiler cannot verify it matches the constraint count.
**How to avoid:** Always ensure the destructuring pattern has exactly as many bindings as constraints. The compiler WILL catch "wrong number of bindings" if you use `let [a, b, c] =` with a 2-element array, but only because array destructuring is checked at compile time. The panic is for the `try_into` on the `Rc<[Rect]>`.
**Warning signs:** Runtime panic with message "invalid number of rects: expected N, found M".

### Pitfall 5: Layout::spacing() Accepts u16 But the Actual Type is Spacing Enum
**What goes wrong:** Attempting to pass a negative number for overlap when using `u16` constants. The `Into<Spacing>` conversion from `u16` always produces `Spacing::Space`, never `Spacing::Overlap`.
**How to avoid:** For this phase, all spacing values are positive (0, 1, 2). No overlap is needed. Define constants as `u16`.

## Current Code Audit: All Spacing/Constraint Values

### Orchestrator draw() -- Lines 1001-1100

| Line | Current Code | Issue | Replacement |
|------|-------------|-------|-------------|
| 1009-1016 | `Layout::default().direction(Direction::Vertical)` | Verbose legacy pattern | `Layout::vertical([...])` |
| 1012 | `Constraint::Min(10)` | Side-effect fill; unclear intent | `Constraint::Fill(1)` |
| 1013-1014 | `Constraint::Length(3)` x2 | Correct; keep as-is | Unchanged |
| 1021-1029 | `Percentage(42)/Length(1)/Percentage(57)` | Sums to 99%+1cell; leaves gaps | `Fill(3)/Fill(4)` + `.spacing(SPACING_NORMAL)` |
| 1028 | `Constraint::Percentage(100)` | Unnecessary single-element layout | Eliminate layout entirely; use `screen_inner` directly |
| 1030-1033 | `Layout::default().direction(Direction::Horizontal).constraints(...).split(...)` | Legacy builder + runtime indexing | `Layout::horizontal([...]).areas()` |
| 1058-1061 | `Percentage(45)/Percentage(55)` horizontal | 100% sum (OK) but inconsistent with Fill pattern | `Fill(2)/Fill(3)` + `.spacing(SPACING_NORMAL)` |
| 1064-1067 | `Length(meta_height)/Min(0)` vertical | `Min(0)` is fill hack | `Length(meta_height)/Fill(1)` |

### Section Renderers -- Ad-Hoc Spacing

| Location | Current Padding | Purpose | Replacement Strategy |
|----------|----------------|---------|---------------------|
| Line 698 | `Padding::new(0, 0, 5, 0)` | Idle text -- 5 rows top padding to visually center | Keep as-is; this is content-specific vertical offset, not layout spacing |
| Line 897 | `Padding::new(1, 1, 0, 0)` | Two-column metadata -- 1 cell left/right padding | Replace with `SPACING_NORMAL` if used via Layout spacing; or keep as block-level padding since this is intra-widget |
| Line 932 | `Padding::new(2, 2, 0, 0)` | Single-column metadata -- 2 cells left/right padding | Replace with `SPACING_SECTION` for consistency; or keep if SECTION spacing (2) matches intent |

**Recommendation on section renderer padding:** The `Padding::new()` values inside section renderers are *intra-widget* padding (content offset within a block), not *inter-section* spacing. The CONTEXT.md says "Apply spacing between artwork/info columns, between metadata/lyrics vertically, around progress bar and controls" -- these are inter-section gaps handled by `Layout::spacing()`. The intra-widget padding should be addressed for consistency but is secondary to the layout spacing system.

Recommend:
1. Replace the `Padding::new(1, 1, 0, 0)` and `Padding::new(2, 2, 0, 0)` in metadata with a consistent value using the spacing constants
2. Keep `Padding::new(0, 0, 5, 0)` in idle text since it serves a different purpose (visual centering)

### Artwork Renderer -- Lines 748-769

The artwork renderer uses manual padding math (`h_padding = 2`, `v_padding = calculated`) for centering. This is Phase 3 scope (LAYT-01: artwork vertical centering). Do NOT modify in Phase 2.

### Controls Renderer -- Lines 961-965

```rust
let btn_width = area.width / controls.len() as u16;
let btn_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(vec![Constraint::Length(btn_width); controls.len()])
    .split(area);
```

This is Phase 3 scope (LAYT-02: button even distribution with Flex). Do NOT modify in Phase 2.

## Constraint Solver Mechanics (Verified from Source)

### Strength Priority (from ratatui-core strengths module)

From highest to lowest priority:

| Strength | Constant | Value | Used By |
|----------|----------|-------|---------|
| REQUIRED/10 | `SPACER_SIZE_EQ` | ~100,000,000 | Spacer equality (Layout::spacing) |
| STRONG*100 | `MIN_SIZE_GE`, `MAX_SIZE_LE` | ~600,000,000 | Min/Max inequality bounds |
| STRONG*10 | `LENGTH_SIZE_EQ` | ~6,000,000 | Length exact size |
| STRONG | `PERCENTAGE_SIZE_EQ` | ~600,000 | Percentage size |
| STRONG/10 | `RATIO_SIZE_EQ` | ~60,000 | Ratio size |
| MEDIUM*10 | `MIN_SIZE_EQ`, `MAX_SIZE_EQ` | ~6,000 | Min/Max equality targets |
| MEDIUM | `FILL_GROW` | ~600 | **Fill grow** |
| MEDIUM/10 | `GROW` | ~60 | Min grow (non-legacy) |
| WEAK*10 | `SPACE_GROW` | ~6 | Spacer grow |
| WEAK | `ALL_SEGMENT_GROW` | ~0.6 | Equal segment grow |

**Key insight for this phase:** `Min` inequality (`MIN_SIZE_GE` at STRONG*100) has FAR higher priority than `Fill` growth (`FILL_GROW` at MEDIUM). This means: if you set `Min(20)` on an artwork segment alongside `Fill(3)`, the minimum of 20 cells is guaranteed. The Fill weight only applies to the remaining space after all Min/Max/Length constraints are satisfied.

### How Fill(3)/Fill(4) Distributes Space

From `configure_fill_constraints()` (line 1052-1081):

The solver adds a proportional constraint: `right_weight * left_size == left_weight * right_size`. For Fill(3)/Fill(4):
- `4 * artwork_size == 3 * info_size`
- Solving: `artwork_size = 3/7 * total`, `info_size = 4/7 * total`
- At 100 cells: artwork=42.8 (~43), info=57.1 (~57) -- rounds to exactly 43+57=100

This eliminates the current 42+1+57=100 pattern where the `Length(1)` separator consumes a cell.

### How Layout::spacing() Works with Fill

When `.spacing(1)` is set on a 2-segment layout:
1. The solver creates 3 spacers: before-first, between, after-last
2. With default `Flex::Start`, only the middle spacer gets the spacing value (1 cell)
3. Available space for segments = total - spacing = total - 1
4. Fill weights distribute the remaining space proportionally

At 100 cells with `spacing(1)`: artwork gets `3/7 * 99 = 42.4 (~42)`, info gets `4/7 * 99 = 56.6 (~57)`, spacing gets 1. Total: 42+1+57=100. Exact same visual result as the current `Percentage(42)/Length(1)/Percentage(57)` but without the fragile 99%-sum arithmetic.

## Code Examples

### Complete Orchestrator Rewrite

```rust
// Source: verified against ratatui-core 0.1.0 solver source + test cases

// Spacing constants (place near top of file with theme constants)
const SPACING_TIGHT: u16 = 0;
const SPACING_NORMAL: u16 = 1;
const SPACING_SECTION: u16 = 2;

// In draw():
let [display_area, tuner_area, control_area] = Layout::vertical([
    Constraint::Fill(1),      // display takes all remaining space
    Constraint::Length(3),    // tuner bar: fixed 3 rows
    Constraint::Length(3),    // control bar: fixed 3 rows
])
.areas(chassis_inner);

let screen_inner = draw_screen_border(f, display_area, theme);
let show_artwork = app.config.artwork.album && display_area.width > 50;

if show_artwork {
    let [artwork_col, info_col] = Layout::horizontal([
        Constraint::Fill(3),  // artwork: 3/7 of space (~43%)
        Constraint::Fill(4),  // info: 4/7 of space (~57%)
    ])
    .spacing(SPACING_NORMAL)
    .areas(screen_inner);

    draw_artwork(f, artwork_col, ...);
    // ... use info_col for metadata/lyrics
} else {
    // No layout needed; info takes full screen_inner
    let info_col = screen_inner;
    // ... use info_col for metadata/lyrics
}
```

### Metadata/Lyrics Split

```rust
let has_lyrics = app.current_lyrics.is_some();
let info_height = info_col.height as usize;
let is_two_columns = show_artwork
    && (metadata_width > 80 || (has_lyrics && info_height <= 14))
    && metadata_width >= 40;
let meta_height = if is_two_columns { 7 } else { 10 };

let (metadata_area, lyrics_area) = if !show_artwork && has_lyrics {
    // No artwork: side-by-side metadata/lyrics
    let [meta, lyrics] = Layout::horizontal([
        Constraint::Fill(2),
        Constraint::Fill(3),
    ])
    .spacing(SPACING_NORMAL)
    .areas(info_col);
    (meta, lyrics)
} else if has_lyrics && info_height > meta_height + 2 {
    // Artwork visible: stacked metadata over lyrics
    let [meta, lyrics] = Layout::vertical([
        Constraint::Length(meta_height as u16),
        Constraint::Fill(1),
    ])
    .spacing(SPACING_NORMAL)
    .areas(info_col);
    (meta, lyrics)
} else {
    (info_col, Rect::default())
};
```

### Metadata Renderer Padding Consistency

```rust
// Two-column mode: use SPACING_NORMAL for left/right padding
Paragraph::new(lines)
    .block(Block::default().padding(ratatui::widgets::Padding::new(
        SPACING_NORMAL, SPACING_NORMAL, 0, 0
    )))

// Single-column mode: use SPACING_SECTION for left/right padding
Paragraph::new(lines)
    .block(Block::default().padding(ratatui::widgets::Padding::new(
        SPACING_SECTION, SPACING_SECTION, 0, 0
    )))
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Layout::default().direction(Direction::Vertical)` | `Layout::vertical([...])` | Ratatui 0.26 | Shorter, clearer intent |
| `Constraint::Min(0)` as "fill remaining" | `Constraint::Fill(1)` | Ratatui 0.26 | Fill communicates intent; Min(0) is a side-effect hack |
| `Constraint::Percentage(N)` pairs summing to 99% | `Constraint::Fill(w1)/Fill(w2)` | Ratatui 0.26 | Exact proportional distribution with no remainder |
| `.split(area)` + `area[0]`, `area[1]` | `.areas::<N>(area)` destructuring | Ratatui 0.26 | Compile-time binding count check |
| Manual `Constraint::Length(1)` separators | `Layout::spacing(1)` | Ratatui 0.26 | Spacing managed by layout engine; eliminates ghost columns |

## Open Questions

1. **Metadata internal padding: SPACING_NORMAL vs SPACING_SECTION?**
   - What we know: Two-column metadata uses `Padding::new(1, 1, 0, 0)` (=1 cell), single-column uses `Padding::new(2, 2, 0, 0)` (=2 cells). SPACING_NORMAL=1, SPACING_SECTION=2.
   - What's unclear: Whether the difference between two-column and single-column padding is intentional (wider padding for wider area) or accidental.
   - Recommendation: Map the existing values to constants: two-column uses `SPACING_NORMAL`, single-column uses `SPACING_SECTION`. This preserves current visual behavior while making it systematic. The planner can decide to unify them if desired.

2. **Should Layout::spacing() be applied to the vertical main layout?**
   - What we know: The vertical split (`display_area/tuner_area/control_area`) currently has no spacing -- the bordered blocks provide visual separation. Adding `spacing(1)` would add 2 cells of unused space (1 between display/tuner, 1 between tuner/controls).
   - What's unclear: Whether the bordered blocks already provide enough visual separation, or if a 1-cell gap between the progress bar and control bar would look better.
   - Recommendation: Do NOT add spacing to the main vertical layout. The bordered blocks (chassis, screen, button borders) already provide visual separation. Adding Layout spacing here would waste vertical space that the display area needs for lyrics.

## Sources

### Primary (HIGH confidence)
- Ratatui-core 0.1.0 source: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/layout.rs` -- solver implementation, strength priorities, Fill/Min interaction, spacing mechanics, test cases
- Ratatui-core 0.1.0 constraint source: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/constraint.rs` -- Constraint enum definition
- Ratatui 0.30.0 prelude source: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-0.30.0/src/prelude.rs` -- re-export verification
- Project source: `src/ui/mod.rs` -- current orchestrator code, all padding/constraint values audited

### Secondary (MEDIUM confidence)
- STACK.md research (2026-03-26) -- Ratatui 0.30 API overview and migration patterns
- FEATURES.md research (2026-03-26) -- Competitor analysis and feature prioritization

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all APIs verified in local ratatui-core source code, not just documentation
- Architecture: HIGH -- replacement patterns verified against solver test cases showing exact input/output
- Pitfalls: HIGH -- all pitfalls derived from reading the actual solver implementation

**Research date:** 2026-03-26
**Valid until:** Stable (ratatui 0.30 is current release; layout solver API is mature)
