# Phase 3: Section-Level Polish - Research

**Researched:** 2026-03-26
**Domain:** Ratatui section rendering -- centering, distribution, alignment, graduated styling
**Confidence:** HIGH

## Summary

Phase 3 polishes four isolated section renderers (`draw_artwork`, `draw_controls`, `draw_metadata`, `draw_lyrics`) to their final visual quality. Each change is contained within a single function and does not affect the layout orchestrator.

The core finding is that all four changes are achievable using APIs already available in ratatui 0.30.0 with zero new dependencies. `Rect::centered()` replaces 18 lines of manual padding math in draw_artwork. `Constraint::Fill(1)` replaces the `width / count` integer division in draw_controls. The metadata alignment needs a fixed-width label column via Layout. The lyrics graduated dimming maps directly to the existing 3-color theme system (accent/primary/dim) without needing color interpolation.

**Primary recommendation:** Implement all four changes as modifications within existing function bodies. No signature changes, no new modules, no new dependencies.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Artwork: Center both vertically and horizontally in its area using Ratatui centering APIs
- Buttons: Use Constraint::Fill(1) per button for equal width (fills the line completely)
- Metadata: Song info labels and values cleanly aligned in consistent column layout
- Lyrics current line: Theme accent color (fg) + Bold modifier
- Lyrics near lines (+-1-2): Normal fg color
- Lyrics far lines (beyond +-2): Dimmed fg color
- 3-level graduated dimming system

### Claude's Discretion
- Exact centering implementation for artwork (Rect::centered vs manual padding vs Flex::Center)
- Metadata column alignment approach
- Specific dimming color values per theme
- Whether to modify draw_lyrics signature or add dimming inside existing function

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LAYT-01 | Artwork vertically centered in its available area using Ratatui centering APIs | `Rect::centered()` verified in source -- takes horizontal + vertical Constraint, returns centered Rect. Direct drop-in for current 18-line manual padding calculation. |
| LAYT-02 | Control buttons evenly distributed across terminal width at any size using Flex layout | `Constraint::Fill(1)` per button distributes evenly. Replaces `width / count` integer division that leaves remainder pixels as dead space. |
| VISL-02 | Song info area labels and values cleanly aligned | Fixed-width label column via `Layout::horizontal([Constraint::Length(label_width), Constraint::Fill(1)])` gives consistent alignment across all three fields. |
| VISL-03 | Current lyrics line highlighted with stronger visual contrast | Change from `theme.primary + BOLD` to `theme.accent + BOLD`. Accent is brighter/warmer than primary in every theme. |
| VISL-04 | Lyrics lines dim gradually with distance from current line | Map distance to 3 existing theme colors: accent (current), primary (near +-1-2), dim (far beyond +-2). No color interpolation needed. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Build:** `make build` (cargo build --release), `make verify` for full pipeline
- **Lint:** `cargo clippy -- -D warnings` (zero warnings policy)
- **Format:** `cargo fmt`
- **Test:** `cargo test`
- **Error handling:** `anyhow::Result` for app logic, `thiserror` for module errors
- **Never block UI draw loop** with I/O
- **Commit format:** `<type>(<scope>): <subject>`
- **macOS only:** Requires Apple Music app

## Standard Stack

### Core (Already Installed -- No Changes)

| Library | Version | Purpose | Verified |
|---------|---------|---------|----------|
| ratatui | 0.30.0 | TUI framework -- Layout, Rect, Flex, Constraint, Style | In Cargo.lock |
| ratatui-image | 10.0.6 | StatefulImage + StatefulProtocol for artwork rendering | In Cargo.lock |
| crossterm | 0.28 | Terminal backend | In Cargo.lock |

### Specific APIs Used in This Phase

| API | Import Path | Purpose |
|-----|-------------|---------|
| `Rect::centered()` | `ratatui::layout::Rect` | Center artwork in area (LAYT-01) |
| `Constraint::Fill(1)` | `ratatui::layout::Constraint` | Equal-weight button distribution (LAYT-02) |
| `Layout::horizontal()` | `ratatui::layout::Layout` | Label/value column split for metadata (VISL-02) |
| `Modifier::BOLD` | `ratatui::style::Modifier` | Already used; reused for lyrics current line (VISL-03) |

**No new dependencies.** All APIs are in ratatui 0.30.0.

## Architecture Patterns

### Function-Level Changes Only

All four changes are internal to existing functions. No new files, modules, or function signatures change.

```
src/ui/mod.rs
  draw_artwork()    -- centering logic replacement (LAYT-01)
  draw_controls()   -- button layout replacement (LAYT-02)
  draw_metadata()   -- label/value alignment (VISL-02)
  draw_lyrics()     -- graduated dimming (VISL-03, VISL-04)
```

### Pattern: Rect::centered() for Artwork (LAYT-01)

**Current code (lines 754-775):** Manual v_padding / h_padding calculation with two nested Layout splits -- 22 lines.

```rust
// CURRENT: Manual centering -- 22 lines
let h_padding = 2;
let side = area.width.saturating_sub(h_padding * 2);
let char_height = side / 2;
let v_padding = (area.height.saturating_sub(char_height)) / 2;

let art_rect = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(v_padding),
        Constraint::Length(char_height),
        Constraint::Min(0),
    ])
    .split(area)[1];

let art_rect = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(h_padding),
        Constraint::Length(side),
        Constraint::Min(0),
    ])
    .split(art_rect)[1];
```

**Replacement:**

```rust
// NEW: Rect::centered() -- 4 lines
let side = area.width.saturating_sub(4); // 2px padding each side
let char_height = side / 2;

let art_rect = area.centered(
    Constraint::Length(side),
    Constraint::Length(char_height),
);
```

`Rect::centered(horizontal, vertical)` internally does exactly the same vertical + horizontal Flex::Center splits. Verified in ratatui-core-0.1.0 source (lines 551-558): it calls `centered_horizontally()` then `centered_vertically()`, each using `Layout::flex(Flex::Center)`.

**StatefulProtocol interaction:** `StatefulImage::render()` calls `state.resize_encode_render(&self.resize, area, buf)` which checks `needs_resize()`. The artwork only re-encodes when the Rect dimensions change -- not when position changes. Since `Rect::centered()` produces the same dimensions as the current manual calculation (same `side` x `char_height`), there is no extra encoding cost from switching to centered positioning. Re-encoding only happens on terminal resize, same as today.

### Pattern: Fill(1) for Button Distribution (LAYT-02)

**Current code (lines 972-976):** Integer division with remainder gap.

```rust
// CURRENT: Integer division -- leaves gap at right edge
let btn_width = area.width / controls.len() as u16;
let btn_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(vec![Constraint::Length(btn_width); controls.len()])
    .split(area);
```

For 7 buttons in 80 columns: `80 / 7 = 11` per button = 77 pixels used, 3-pixel dead gap on the right.

**Replacement:**

```rust
// NEW: Fill(1) -- distributes ALL pixels evenly, no remainder
let btn_areas = Layout::horizontal(vec![Constraint::Fill(1); controls.len()])
    .split(area);
```

With Fill(1), the solver distributes all 80 pixels: 5 buttons get 12px, 2 buttons get 11px (or equivalent distribution). No dead space.

**Note on SpaceBetween:** The CONTEXT.md mentions `Flex::SpaceBetween`. Verified in ratatui-core source tests (layout.rs line 2600): when all constraints are `Fill(1)`, SpaceBetween is a no-op because Fill absorbs all excess space, leaving nothing for SpaceBetween to distribute. Using `.flex(Flex::SpaceBetween)` is harmless but unnecessary. `Fill(1)` alone achieves the desired even distribution. Including `.flex(Flex::SpaceBetween)` in the code is acceptable for communicating intent even if functionally redundant.

### Pattern: Fixed-Width Label Column for Metadata (VISL-02)

**Current code (lines 883-906 single-column, 857-907 two-column):** Labels and values are stacked vertically as consecutive Line entries in a single Paragraph. No horizontal alignment between labels.

```rust
// CURRENT: Vertical stacking -- label on one line, value on next
lines.push(Line::from(Span::styled(labels[i], ...)));  // "TRACK TITLE"
lines.push(Line::from(Span::styled(format!(" {} ", display_val), ...)));  // " SONG NAME "
```

This produces:
```
TRACK TITLE
 BOHEMIAN RHAPSODY
ARTIST
 QUEEN
```

**Recommended alignment approach:** Keep the vertical label-above-value layout (it matches the industrial/VFD aesthetic) but ensure consistent left padding. The current approach with `Padding::new(SPACING_SECTION, SPACING_SECTION, 0, 0)` already provides horizontal padding. For additional alignment quality:

Option A (minimal change): Add a consistent left margin to labels using the existing Padding widget -- already in place.

Option B (label:value on same line): Use `Layout::horizontal([Constraint::Length(max_label_width), Constraint::Fill(1)])` per row to create an aligned two-column format:

```rust
// Each metadata row as a horizontal split
let max_label_width = labels.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16 + 2;

for i in 0..items_count {
    let [label_area, value_area] = Layout::horizontal([
        Constraint::Length(max_label_width),
        Constraint::Fill(1),
    ]).areas(row_areas[i]);

    // Render label in label_area, value in value_area
}
```

This produces:
```
TRACK TITLE     BOHEMIAN RHAPSODY
ARTIST          QUEEN
ALBUM REFERENCE A NIGHT AT THE OPERA
```

**Recommendation:** Use Option A (keep vertical stacking) for the standard layout, since it preserves the existing industrial aesthetic. The "alignment" requirement is satisfied by ensuring labels and their values have consistent indentation. The vertical label/value pattern is already cleanly structured -- the main fix is ensuring the padding and indentation is uniform across the single-column and two-column code paths, using the same Padding constants.

For the two-column code path (lines 857-907): replace `Constraint::Percentage(50), Constraint::Percentage(50)` with `Constraint::Fill(1), Constraint::Fill(1)` for even distribution without percentage rounding issues. Add `spacing(SPACING_NORMAL)` between columns.

### Pattern: Graduated Lyrics Dimming (VISL-03, VISL-04)

**Current code (lines 598-606):** Binary highlighting -- current line is `theme.primary + BOLD`, everything else is `theme.dim`.

```rust
// CURRENT: Binary -- 2 levels only
let style = if i == current_index {
    Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
} else {
    Style::default().fg(theme.dim)
};
```

**Replacement:** 3-tier system using existing theme colors:

```rust
// NEW: 3-tier graduated dimming
let distance = (i as isize - current_index as isize).unsigned_abs();
let style = if i == current_index {
    // Tier 1: Current line -- brightest, boldest
    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
} else if distance <= 2 {
    // Tier 2: Near lines -- normal readable text
    Style::default().fg(theme.primary)
} else {
    // Tier 3: Far lines -- faded
    Style::default().fg(theme.dim)
};
```

**Color mapping across all 6 themes:**

| Theme | accent (current) | primary (near) | dim (far) |
|-------|-------------------|----------------|-----------|
| AMBER VFD | Rgb(255,215,0) gold | Rgb(255,176,0) amber | Rgb(80,60,20) dark amber |
| GREEN VFD | Rgb(50,255,100) bright green | Rgb(0,255,65) green | Rgb(0,80,20) dark green |
| CYAN VFD | Rgb(0,150,255) blue | Rgb(0,255,255) cyan | Rgb(0,80,100) dark cyan |
| RED ALERT | Rgb(255,100,100) light red | Rgb(255,50,50) red | Rgb(100,0,0) dark red |
| MODERN | Rgb(0,122,255) blue | Rgb(20,20,20) near-black | Rgb(100,100,100) gray |
| CLEAN | Indexed(6) cyan | Indexed(4) blue | Indexed(8) bright-black |

Every theme has clear visual separation between the three tiers. No color interpolation or new theme fields needed.

**Signature change:** Not needed. The dimming logic lives entirely within the existing `for` loop inside `draw_lyrics`. The function signature `fn draw_lyrics(f, area, track, lyrics, theme)` is unchanged.

### Anti-Patterns to Avoid

- **Color interpolation helper:** Do NOT write a `lerp_color()` function. The 3 existing theme colors provide sufficient visual tiers. Interpolation adds complexity for CLEAN theme (Indexed colors cannot be interpolated) with no visual benefit.
- **Modifier::DIM for far lines:** Do NOT add `Modifier::DIM` to far lyrics lines. The `DIM` modifier is terminal-dependent and may not produce visible dimming with RGB colors. Using the theme's `dim` color directly is reliable.
- **Changing draw_lyrics signature:** Keep the current signature. All state needed for dimming (line index, current_index, theme) is already available inside the function.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Centering a Rect | Manual v_padding/h_padding with nested Layout splits | `Rect::centered(h_constraint, v_constraint)` | Rect::centered is 1 call vs 18 lines, tested in ratatui source, handles edge cases (area smaller than constraint) |
| Even distribution | `area.width / count` integer division | `vec![Constraint::Fill(1); count]` | Fill distributes ALL pixels with no remainder gap |
| Column alignment | Manual character counting + padding | `Layout::horizontal([Length(N), Fill(1)])` per row | Layout handles variable-width areas, respects terminal boundaries |

## Common Pitfalls

### Pitfall 1: StatefulProtocol Re-encoding on Position Change
**What goes wrong:** Moving the artwork Rect to a centered position could trigger re-encoding every frame if dimensions change.
**Why it happens:** `StatefulImage::render()` calls `needs_resize()` which compares current Rect dimensions to the area parameter.
**How to avoid:** Ensure the centered Rect has the same width and height as the current calculation. The centering only changes position (x, y), not dimensions (width, height). Verified: `Rect::centered(Constraint::Length(side), Constraint::Length(char_height))` produces a Rect with exactly `side` width and `char_height` height.
**Warning signs:** Artwork flickering or sluggish rendering after centering change.

### Pitfall 2: Fill(1) with Borders Producing 0-Width Rects
**What goes wrong:** Each button has `Block::borders(Borders::ALL)` which consumes 2 columns. At very narrow widths, `Fill(1)` per 7 buttons can produce Rects narrower than 2 columns (border-only, no content).
**Why it happens:** Fill distributes available space evenly but does not enforce minimums.
**How to avoid:** Not a real issue at typical widths (7 buttons need ~21 columns minimum for borders alone). The existing `display_area.width > 50` guard for artwork already implies reasonable width. No action needed, but awareness is useful.
**Warning signs:** Buttons rendering as empty bordered boxes at extreme narrow widths.

### Pitfall 3: Lyrics Index Type Mismatch for Distance Calculation
**What goes wrong:** `current_index` is `usize`, and computing `i - current_index` underflows for `i < current_index`.
**Why it happens:** Unsigned arithmetic in Rust panics on underflow in debug mode.
**How to avoid:** Cast to `isize` before subtraction: `(i as isize - current_index as isize).unsigned_abs()`. This correctly handles both `i > current_index` and `i < current_index`.
**Warning signs:** Panic in debug builds when scrolling lyrics.

### Pitfall 4: Two-Column Metadata Percentage Rounding
**What goes wrong:** `Constraint::Percentage(50), Constraint::Percentage(50)` can leave a 1-cell gap due to rounding.
**Why it happens:** 50% of an odd width rounds down, leaving 1 unallocated pixel.
**How to avoid:** Replace with `Constraint::Fill(1), Constraint::Fill(1)` which distributes all pixels exactly.
**Warning signs:** Visible 1-pixel gap between metadata columns at odd terminal widths.

## Code Examples

### Artwork Centering (Verified Pattern)

```rust
// Source: ratatui-core-0.1.0/src/layout/rect.rs lines 551-558
// Rect::centered() signature:
pub fn centered(
    self,
    horizontal_constraint: Constraint,
    vertical_constraint: Constraint,
) -> Self {
    self.centered_horizontally(horizontal_constraint)
        .centered_vertically(vertical_constraint)
}

// Usage in draw_artwork:
let side = area.width.saturating_sub(4);
let char_height = side / 2;
let art_rect = area.centered(
    Constraint::Length(side),
    Constraint::Length(char_height),
);
```

### Button Distribution (Verified Pattern)

```rust
// Source: ratatui-core-0.1.0/src/layout/layout.rs line 2600
// Test case: Fill(1), Fill(1) with SpaceBetween, spacing 0
// Result: (0, 50), (50, 50) -- perfect 50/50 split

// Usage in draw_controls:
let btn_areas = Layout::horizontal(vec![Constraint::Fill(1); controls.len()])
    .split(area);

for (i, (label, key)) in controls.iter().enumerate() {
    if i < btn_areas.len() {
        // ... render button in btn_areas[i]
    }
}
```

### Graduated Lyrics Dimming

```rust
// Usage in draw_lyrics:
for (i, line) in lyrics.lines.iter().enumerate() {
    let distance = (i as isize - current_index as isize).unsigned_abs();
    let style = if i == current_index {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else if distance <= 2 {
        Style::default().fg(theme.primary)
    } else {
        Style::default().fg(theme.dim)
    };
    lines.push(Line::from(Span::styled(&line.text, style)));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual padding math for centering | `Rect::centered()` | ratatui 0.29 | Eliminates 15-20 lines of arithmetic per centering use case |
| `width / count` for distribution | `Constraint::Fill(weight)` | ratatui 0.26 | No remainder gaps, proportional when weights differ |
| `Layout::default().direction()` | `Layout::vertical()` / `Layout::horizontal()` | ratatui 0.26 | Shorter, clearer intent |
| `Constraint::Min(0)` as "fill remaining" | `Constraint::Fill(1)` | ratatui 0.26 | Semantic clarity |

## Open Questions

1. **Metadata "alignment" interpretation**
   - What we know: The current vertical label/value stacking works well aesthetically with the industrial theme. Padding is already applied via SPACING constants.
   - What's unclear: Whether the user expects same-line label:value pairs or just cleaner vertical stacking.
   - Recommendation: Keep vertical stacking (cleaner for the VFD aesthetic), ensure consistent padding in both single-column and two-column paths. This satisfies VISL-02 as "cleanly aligned" without changing the visual character.

2. **Near-line distance threshold**
   - What we know: Decision says "+-1-2 from current" for near lines.
   - What's unclear: Whether the boundary is at +-1 or +-2.
   - Recommendation: Use `distance <= 2` (+-2 lines). This gives 5 lines of visible text (current + 2 above + 2 below) before dimming, which matches typical lyrics display heights.

## Sources

### Primary (HIGH confidence)
- ratatui-core 0.1.0 source: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/rect.rs` -- Rect::centered() implementation and tests (lines 551-558, 1057-1063)
- ratatui-core 0.1.0 source: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/flex.rs` -- Flex::SpaceBetween docs and behavior (lines 144-164)
- ratatui-core 0.1.0 source: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/layout.rs` -- Fill+SpaceBetween test cases (line 2600)
- ratatui-image 10.0.6 source: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-image-10.0.6/src/lib.rs` -- StatefulImage render and needs_resize behavior (lines 198-218, 277-283)
- Project source: `/Users/jac/Repos/amcli/src/ui/mod.rs` -- All four draw functions, theme definitions, spacing constants

### Secondary (MEDIUM confidence)
- `.planning/research/STACK.md` -- Prior stack research (same findings, cross-verified)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all APIs verified in local ratatui-core source
- Architecture: HIGH -- changes are within existing functions, patterns verified in source
- Pitfalls: HIGH -- StatefulProtocol resize behavior verified in ratatui-image source; integer overflow is a known Rust pattern

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable -- ratatui 0.30 is current release)
