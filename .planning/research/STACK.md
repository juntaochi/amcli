# Technology Stack: Ratatui Responsive Layout & Visual Polish

**Project:** AMCLI (Apple Music TUI)
**Researched:** 2026-03-26
**Focus:** Layout primitives, centering, proportional distribution, adaptive layouts
**Current versions:** ratatui 0.30.0, crossterm 0.28, ratatui-image 10.0

## Recommended Stack

### Core Layout APIs (Already Available -- No New Dependencies)

Ratatui 0.30.0 ships with everything needed. The project already has the right version.

| API | Module | Purpose | Why |
|-----|--------|---------|-----|
| `Layout::vertical()` / `Layout::horizontal()` | `ratatui::layout` | Ergonomic constructors replacing `Layout::default().direction(...)` | Shorter, clearer intent. Returns `Layout` directly with direction set. |
| `Layout::areas::<N>()` | `ratatui::layout` | Compile-time-checked split into `[Rect; N]` | Eliminates runtime indexing errors; panics at compile-time if N mismatches constraint count. |
| `Rect::centered()` | `ratatui::layout` | Center a rect both horizontally and vertically | Direct replacement for the manual v_padding/h_padding centering pattern currently used for artwork. |
| `Rect::centered_horizontally()` / `Rect::centered_vertically()` | `ratatui::layout` | Single-axis centering | For cases where only one axis needs centering (e.g., button bar). |
| `Constraint::Fill(weight)` | `ratatui::layout` | Proportional space distribution | Replaces `Constraint::Min(0)` as a "take remaining space" marker. Fill with weights enables proportional sharing between multiple flexible areas. |
| `Flex` enum | `ratatui::layout` | Space distribution strategy | `Flex::SpaceBetween` for button distribution. `Flex::Center` for centering groups. Already used internally by `Rect::centered*()`. |
| `Spacing` enum | `ratatui::layout` | Gaps or overlaps between layout segments | `Spacing::Space(1)` for uniform gaps. `Spacing::Overlap(1)` for border-sharing between adjacent blocks. |

**Confidence:** HIGH -- all verified in ratatui-core 0.1.0 source code at `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/`.

### Macros (Already Available -- Transitive Dependency)

`ratatui-macros 0.7.0` is already pulled in by ratatui 0.30.0 and re-exported via `ratatui::macros`. The `macros` feature is enabled by default.

| Macro | Purpose | Why Use It |
|-------|---------|------------|
| `vertical![constraints]` | Shorthand for `Layout::vertical([...])` | Concise constraint syntax: `vertical![==42%, ==1, *=1]` |
| `horizontal![constraints]` | Shorthand for `Layout::horizontal([...])` | Same benefits for horizontal splits |
| `constraints![...]` | Shorthand constraint array | Use when building constraints dynamically or passing to `Layout::new()` |

**Constraint shorthand notation:**
- `==50` -- `Constraint::Length(50)`
- `==30%` -- `Constraint::Percentage(30)`
- `==1/3` -- `Constraint::Ratio(1, 3)`
- `>=3` -- `Constraint::Min(3)`
- `<=10` -- `Constraint::Max(10)`
- `*=1` -- `Constraint::Fill(1)`

**Recommendation:** Use macros selectively. They help most in complex nested layouts where readability matters. For simple 2-3 constraint layouts, `Layout::vertical([...])` with explicit `Constraint::` variants is clearer because it is self-documenting.

**Confidence:** HIGH -- re-export verified via docs.rs for ratatui 0.30.0. Transitive dependency confirmed in Cargo.lock.

### Additional Rect Methods (New in 0.30)

| Method | Purpose | Replaces |
|--------|---------|----------|
| `Rect::layout::<N>(&Layout)` | Split rect into `[Rect; N]` via Layout | `Layout::split(area)[index]` pattern |
| `Rect::layout_vec(&Layout)` | Split rect into `Vec<Rect>` | Same, but for dynamic constraint counts |
| `Rect::inner(Margin)` | Shrink rect by margin | Already used via `Block::inner()`, but useful without a Block widget |
| `Rect::outer(Margin)` | Expand rect by margin | New; useful for computing borders |
| `Rect::rows()` / `Rect::columns()` | Iterate 1-cell-high rows or 1-cell-wide columns | Manual row iteration loops |

**Confidence:** HIGH -- verified in source.

## Migration Patterns for Current Code

### 1. Artwork Centering (Replace Manual Padding Math)

**Current (lines 720-744):** Manual v_padding / h_padding calculation with nested Layout splits.

**Replace with:**
```rust
// Calculate the artwork dimensions (keep this logic)
let side = artwork_column.width.saturating_sub(4); // 2px padding each side
let char_height = side / 2;

// Center the artwork rect in one call
let art_rect = artwork_column.centered(
    Constraint::Length(side),
    Constraint::Length(char_height),
);
```

This is a direct drop-in. `Rect::centered()` internally uses `Layout::horizontal/vertical` with `Flex::Center`, producing identical results to the manual padding calculation but in 4 lines instead of 18.

### 2. Button Distribution (Replace Fixed-Width Slicing)

**Current (lines 1028-1032):** `btn_width = area.width / controls.len()` then `vec![Constraint::Length(btn_width); N]`.

**Replace with:**
```rust
let btn_layout = Layout::horizontal(vec![Constraint::Fill(1); controls.len()])
    .flex(Flex::SpaceBetween)
    .spacing(1)
    .areas::<7>(control_area); // or .split() for dynamic count
```

`Constraint::Fill(1)` gives each button equal weight. `Flex::SpaceBetween` distributes leftover space between buttons rather than leaving dead space at the right edge. `spacing(1)` adds 1-cell gaps. This naturally fills the full width at any terminal size.

For dynamic button count, use `.split()` instead of `.areas::<N>()`.

### 3. Main Content Split (Replace Percentage Guessing)

**Current (lines 701-714):** `Constraint::Percentage(42)` / `Constraint::Percentage(57)` with a 1-cell gap.

**Replace with:**
```rust
let [artwork_col, info_col] = Layout::horizontal([
    Constraint::Fill(2),  // artwork gets 2/5 of space
    Constraint::Fill(3),  // info gets 3/5 of space
])
.spacing(1)
.areas(screen_inner);
```

`Constraint::Fill` with weights is clearer intent than percentages that do not add to 100%. The `spacing(1)` replaces the manual `Constraint::Length(1)` separator column.

### 4. Outer Frame Split (Replace Default Builder)

**Current (lines 673-680):** `Layout::default().direction(Direction::Vertical).constraints([...]).split(...)`.

**Replace with:**
```rust
let [display_area, tuner_area, control_area] =
    Layout::vertical([
        Constraint::Fill(1),      // display takes remaining space
        Constraint::Length(3),    // tuner bar: fixed 3 rows
        Constraint::Length(3),    // control bar: fixed 3 rows
    ])
    .areas(chassis_inner);
```

`Layout::vertical()` replaces `.direction(Direction::Vertical)`. `Constraint::Fill(1)` replaces `Constraint::Min(10)` -- Fill is the idiomatic 0.30 way to say "take all remaining space." The `.areas::<3>()` gives compile-time destructuring.

## Constraint Priority (Reference)

When constraints conflict, the solver resolves them in this priority order:

1. **Min** -- always honored first
2. **Max** -- always honored second
3. **Length** -- fixed sizes next
4. **Percentage** -- proportional allocation
5. **Ratio** -- fractional allocation
6. **Fill** -- lowest priority, distributes leftovers

This means `Fill` never starves a `Length` or `Min` constraint. Use this to build layouts where fixed-size elements (progress bar, control bar) always get their space and flexible elements (artwork, lyrics) share the remainder.

## What NOT to Use

| Avoid | Use Instead | Why |
|-------|-------------|-----|
| `Layout::default().direction(Direction::Vertical)` | `Layout::vertical([...])` | Verbose; the constructor is the 0.30 idiom |
| `Constraint::Min(0)` as "fill remaining" | `Constraint::Fill(1)` | `Fill` communicates intent. `Min(0)` is a legacy pattern that means "minimum zero height" which happens to fill space as a side effect. |
| `Constraint::Percentage` pairs that add to 99% | `Constraint::Fill` with weights | Percentages rounding to 99% leaves a 1-cell gap. Fill distributes exactly. |
| Manual padding math for centering | `Rect::centered()` family | The manual approach is 15+ lines of arithmetic that `centered()` encapsulates in one call. |
| `Layout::split()` when count is known | `Layout::areas::<N>()` or `Rect::layout::<N>()` | Compile-time checked. No runtime `[index]` on `Rc<[Rect]>`. |
| `ratatui-macros` as a direct dependency | Already available via `ratatui::macros` | It is re-exported. Adding it separately would be a duplicate dependency. |
| Third-party layout crates | Built-in Layout + Flex | Ratatui 0.30's layout system handles all the project's needs. Extra crates add complexity without benefit. |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Layout engine | Built-in `Layout` + `Flex` + `Rect` helpers | `tui-layout` or custom constraint solver | Ratatui 0.30 covers centering, proportional distribution, and adaptive layouts natively. No gaps to fill. |
| Macro sugar | `ratatui::macros` (built-in) | `ratatui-macros` as explicit dep | Already re-exported by ratatui 0.30. Adding it separately creates a duplicate. |
| Button distribution | `Flex::SpaceBetween` + `Fill(1)` | Manual width division | Manual division leaves remainder pixels as dead space on the right; Flex distributes evenly. |
| Border overlap | `Spacing::Overlap(1)` | Manual Rect manipulation | Spacing::Overlap is purpose-built for shared-border layouts (e.g., adjacent button blocks). |
| Scrollable lyrics | Built-in `Paragraph` with scroll offset | `tui-scrollview` | The lyrics area does not need virtual scrolling -- it is a fixed viewport with a moving highlight. Paragraph with calculated offset handles this. |
| Visual effects | `tachyonfx` | None | Out of scope for this milestone. Layout and spacing only. |

## No New Dependencies Required

The project constraint says "no new dependencies" and this research confirms none are needed. Every recommended API is available in ratatui 0.30.0 which is already installed. The `ratatui-macros` crate is a transitive dependency (0.7.0) already resolved in Cargo.lock.

**Summary of available-but-unused APIs in the current codebase:**

| API | Status | Location in Source |
|-----|--------|-------------------|
| `Layout::vertical()` / `Layout::horizontal()` | Available, not used | `ratatui::layout::Layout` |
| `Layout::areas::<N>()` | Available, not used | `ratatui::layout::Layout` |
| `Rect::centered()` | Available, not used | `ratatui::layout::Rect` |
| `Rect::centered_horizontally()` | Available, not used | `ratatui::layout::Rect` |
| `Rect::centered_vertically()` | Available, not used | `ratatui::layout::Rect` |
| `Rect::layout::<N>()` | Available, not used | `ratatui::layout::Rect` |
| `Constraint::Fill(u16)` | Available, not used | `ratatui::layout::Constraint` |
| `Flex::SpaceBetween` | Available, not used | `ratatui::layout::Flex` |
| `Flex::Center` | Available, not used (directly) | `ratatui::layout::Flex` |
| `Spacing::Space(u16)` | Available, not used | `ratatui::layout::Spacing` |
| `Spacing::Overlap(u16)` | Available, not used | `ratatui::layout::Spacing` |

## Alignment Type Migration Note

Ratatui 0.30 renamed `Alignment` to `HorizontalAlignment` and added `VerticalAlignment`. The old `Alignment` name still works as a type alias and is not deprecated. No migration is required, but new code should prefer `HorizontalAlignment` for clarity. The current codebase uses `Alignment::Center` in ~8 locations -- these continue to compile without changes.

**Confidence:** HIGH -- verified via [PR #1735](https://github.com/ratatui/ratatui/pull/1735) and confirmed the type alias still resolves in 0.30.0.

## Sources

- [Ratatui Layout Concepts](https://ratatui.rs/concepts/layout/) -- Official guide to the layout system
- [Ratatui Layout API (docs.rs)](https://docs.rs/ratatui/latest/ratatui/layout/struct.Layout.html) -- Full Layout struct documentation
- [Ratatui Rect API (docs.rs)](https://docs.rs/ratatui/latest/ratatui/layout/struct.Rect.html) -- Rect methods including centered*
- [Ratatui Constraint API (docs.rs)](https://docs.rs/ratatui/latest/ratatui/layout/enum.Constraint.html) -- Constraint variants and priority
- [Ratatui Flex Example](https://ratatui.rs/examples/layout/flex/) -- Interactive Flex variant demo
- [Ratatui Center Widget Recipe](https://ratatui.rs/recipes/layout/center-a-widget/) -- Official centering patterns
- [Ratatui Grid Layout Recipe](https://ratatui.rs/recipes/layout/grid/) -- Grid layout patterns
- [Ratatui 0.30 Highlights](https://ratatui.rs/highlights/v030/) -- Version 0.30 new features
- [Ratatui Spacing API (docs.rs)](https://docs.rs/ratatui/latest/ratatui/layout/enum.Spacing.html) -- Spacing/Overlap enum
- [ratatui-macros README](https://github.com/ratatui/ratatui-macros/blob/main/README.md) -- Macro syntax reference
- [ratatui-macros API (docs.rs)](https://docs.rs/ratatui-macros/latest/ratatui_macros/) -- Full macro documentation
- [Alignment rename PR #1735](https://github.com/ratatui/ratatui/pull/1735) -- HorizontalAlignment migration context
- Local source verification: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/rect.rs`
- Local source verification: `/Users/jac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.0/src/layout/layout.rs`
