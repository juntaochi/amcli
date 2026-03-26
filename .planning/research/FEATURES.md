# Feature Research

**Domain:** TUI music player layout and visual polish
**Researched:** 2026-03-26
**Confidence:** HIGH

## Feature Landscape

This research is scoped to the layout/visual polish milestone -- not new functionality. The question is: what layout and visual behaviors make a TUI music player feel polished vs. broken?

Evidence sources: rmpc (Ratatui 0.30 MPD client), spotify-player (Rust/Ratatui Spotify client), termusic (Rust TUI player), ncmpcpp (the gold standard terminal music player), LyricsMPRIS-Rust (lyrics-focused TUI), and the Ratatui 0.30 layout API documentation.

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = the UI looks broken or unfinished.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Artwork vertical centering** | Art pinned to top-left looks like a layout bug. Every polished TUI player (rmpc, spotify-player) centers artwork in its area. | LOW | Ratatui 0.30 has `Rect::centered()` and `Rect::centered_vertically()` -- replaces the manual v_padding calculation currently in the draw function (lines 725-735). One-liner replacement. |
| **Even button distribution** | Buttons bunched to the left with dead space on the right looks broken. Current code divides width evenly by count (`btn_width = area.width / controls.len()`) but renders with `Constraint::Length`, leaving a remainder gap on the right. | LOW | Use `Layout::horizontal([...]).flex(Flex::SpaceBetween)` or `Flex::SpaceEvenly` with `Constraint::Length(btn_width)` per button. Ratatui 0.30 Flex handles the remainder distribution automatically. |
| **Consistent spacing between sections** | Uneven gaps between artwork/info/lyrics/controls look unprofessional. The current layout uses ad-hoc padding values (Padding::new(1,1,0,0), Padding::new(2,2,0,0), Padding::new(0,0,5,0)) with no system. | MEDIUM | Establish a spacing constant (e.g., 1 cell between sibling sections, 2 cells padding within blocks). Apply uniformly through all layout splits. Requires auditing every padding/margin in draw(). |
| **Metadata area clean alignment** | Labels (track/artist/album) should align consistently. Current code uses inline format strings with inconsistent spacing between label and value rows. | LOW | Standardize the vertical rhythm: 1 blank line between label-value groups. Use consistent left padding. The existing scroll_text() handles overflow well -- the alignment is the issue. |
| **Progress bar fills full width** | The gauge already spans the tuner_area width, but the border and label styling should match the visual weight of other sections. | LOW | Already implemented. Minor polish: ensure the gauge borders use consistent border_type with the rest of the theme. |
| **No dead zones at any size** | Empty black areas where content could fill make the layout feel incomplete. The current Constraint::Min(10) for display_area is good, but the 42%/57% artwork split leaves 1% as a separator, and the info area can have large empty regions below metadata when no lyrics are present. | MEDIUM | Use `Constraint::Fill(1)` for flexible areas instead of fixed percentages. When no lyrics exist, the metadata area should vertically center within the available space rather than pin to top. |

### Differentiators (Competitive Advantage)

Features that elevate beyond "not broken" to "this looks professional." These are what separate rmpc/spotify-player from basic TUI apps.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Lyrics current-line highlight with visual weight** | Current implementation bolds the active line and dims others (lines 599-605). Polished players like LyricsMPRIS-Rust and rmpc use a combination of bold + color contrast + surrounding dim to create a "spotlight" effect. The active line should visually pop, not just be slightly different. | LOW | Increase contrast ratio: active line uses theme.accent (brightest color) + BOLD, surrounding lines use graduated dimming (lines near current = theme.dim, lines far away = even dimmer). This is a style-only change. |
| **Lyrics vertical centering (current line at midpoint)** | The existing code scrolls lyrics so the current line lands near the middle (line 609: `current_index.saturating_sub(mid)`). This is correct behavior. The differentiator is making this feel smooth -- avoiding jarring jumps when lines change. | LOW | Already implemented via scroll offset. Improvement: add 1-2 lines of padding above/below the lyrics content so the first and last lines are not pressed against the area edges. |
| **Adaptive artwork/info split ratio** | Fixed 42%/57% split works for typical sizes but wastes space at extreme widths. Polished players adapt the ratio: at narrow widths, artwork gets less space; at wide widths, artwork stays a reasonable fixed size rather than growing proportionally. | MEDIUM | Replace `Constraint::Percentage(42)` with `Constraint::Max(artwork_max)` combined with `Constraint::Min(artwork_min)`. Use a calculated max based on terminal height (artwork should be roughly square in cell units, so max width = height * 2). |
| **Proportional vertical layout** | The current 3-chunk vertical split uses `Constraint::Min(10)` for display and `Constraint::Length(3)` each for progress and controls. At tall terminals this wastes nothing (Min expands). The differentiator is ensuring the display area distributes internal space well between metadata and lyrics. | MEDIUM | When lyrics are present, use `Constraint::Length(meta_lines)` for metadata (exact fit, no excess) and `Constraint::Fill(1)` for lyrics (takes remaining space). When no lyrics, center metadata vertically in the full display area. |
| **Button text truncation at narrow widths** | At narrow terminal widths, buttons overflow or get clipped. Polished apps show abbreviated labels or hide key hints while keeping the button functional. | LOW | Calculate available width per button. If < threshold (e.g., 8 chars), hide the `[key]` hint. If < smaller threshold (e.g., 5 chars), use icon-only (the existing unicode glyphs like the play symbol). |
| **Separator line between artwork and info** | The current layout allocates `Constraint::Length(1)` for a separator column between artwork and info, but nothing is rendered there -- it is just empty space. A subtle vertical line or dim border adds visual structure. | LOW | Render a dim vertical line character (or thin Block border) in the 1-cell separator. Alternatively, use the artwork Block's right border. Minimal code. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem valuable but conflict with this project's constraints or create disproportionate complexity.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Responsive collapse/hide at tiny sizes** | "Hide artwork when terminal is narrow" seems smart. | PROJECT.md explicitly declares this out of scope: "user wants it to work at any size, not degrade." Collapse logic adds branching complexity to every layout calculation. The show_artwork threshold at width > 50 already exists and is sufficient. | Keep the existing width > 50 threshold for artwork. Below that, artwork hides and info takes full width. No further degradation tiers needed. |
| **Animated transitions between layout states** | Smooth resize, fade effects, slide animations. | Terminal rendering is cell-based at 50ms refresh. Animations require frame interpolation, easing functions, and state tracking between draws. Massive complexity for barely-perceptible effect in a 20fps TUI. | Instant layout recalculation on resize. The 50ms poll loop already provides responsive feel. |
| **Pixel-perfect alignment across terminal emulators** | "Works identically in iTerm2, Kitty, Alacritty, Terminal.app." | Character cell dimensions vary by terminal and font. Artwork protocols (Sixel, Kitty, halfblocks) have fundamentally different rendering behaviors. Perfect pixel alignment is impossible. | Use cell-based alignment (Ratatui's natural unit). Accept that artwork rendering varies by protocol. The centering helpers work in cell units, which is the right abstraction level. |
| **Dynamic font-size-aware scaling** | "Detect font size and scale layout accordingly." | Terminals do not reliably expose font metrics. Even terminals that support XTWINOPS or cell-size queries have inconsistent implementations. | Design for cell units. The layout already adapts to terminal dimensions in cells, which is correct. |
| **Configurable layout (user-defined panel arrangement)** | "Let users rearrange panels like tmux." | Massive implementation effort. Requires a layout DSL, constraint solver integration, serialization, and validation. rmpc supports this but it is their core differentiator backed by months of work. | Ship a single well-designed layout. The existing artwork-left/info-right split with vertical metadata/lyrics is the standard pattern. Polish it instead of making it configurable. |
| **Scrollbar widget for lyrics** | "Add a visible scrollbar to show position in lyrics." | Lyrics auto-scroll with playback position -- users do not manually scroll. A scrollbar adds visual noise for no functional benefit. The current-line highlight already communicates position. | The bold/bright current line is the position indicator. No scrollbar needed. |

## Feature Dependencies

```
[Consistent spacing system]
    |
    +--enables--> [Artwork vertical centering] (uses spacing constants for padding)
    +--enables--> [Metadata clean alignment] (uses spacing constants for label/value gaps)
    +--enables--> [No dead zones] (spacing system prevents ad-hoc padding that creates gaps)
    +--enables--> [Button distribution] (spacing between buttons becomes part of the system)

[Artwork vertical centering]
    +--independent (Rect::centered_vertically, no deps)

[Even button distribution]
    +--independent (Flex::SpaceBetween, no deps)

[Lyrics current-line highlight]
    +--independent (style change only)

[Lyrics vertical centering]
    +--enhances--> [Lyrics current-line highlight] (better centering makes the highlight feel more deliberate)

[Adaptive artwork/info split]
    +--requires--> [Consistent spacing system] (split ratio must account for consistent padding)
    +--requires--> [Artwork vertical centering] (centered artwork is prerequisite to adaptive sizing)

[Proportional vertical layout]
    +--requires--> [Consistent spacing system]
    +--enhances--> [No dead zones]

[Button text truncation]
    +--requires--> [Even button distribution] (must distribute first, then truncate within each slot)
```

### Dependency Notes

- **Consistent spacing system is foundational:** It is not a visible feature itself, but every other layout improvement builds on having predictable, uniform spacing values. Do this first.
- **Artwork centering and button distribution are independent leaf nodes:** These can be done in any order, with no prerequisites beyond the spacing system.
- **Adaptive artwork split requires centering first:** You need centering working correctly before you tune the ratio, because the centering behavior changes how artwork occupies its area.
- **Button truncation requires distribution first:** You must know each button's allocated width before deciding what to truncate.

## MVP Definition

### Must Complete (This Milestone)

These are the active requirements from PROJECT.md. All are layout/polish; none add new features.

- [ ] Consistent spacing system -- establish constants, audit all padding/margin values
- [ ] Artwork vertical centering -- use `Rect::centered_vertically()` or `Rect::centered()`
- [ ] Button even distribution -- use `Layout::horizontal().flex(Flex::SpaceBetween)`
- [ ] Metadata area clean alignment -- standardize label/value spacing
- [ ] Lyrics presentation with better highlight -- increase contrast on current line
- [ ] No dead zones -- fill available space proportionally, center content when area is larger than needed

### Add After Core Layout (Stretch Goals)

Features that improve polish further but are not required for the milestone to succeed.

- [ ] Adaptive artwork/info split ratio -- replace fixed percentages with Min/Max constraints
- [ ] Button text truncation at narrow widths -- graceful degradation of button labels
- [ ] Separator line between artwork and info -- visual structure in the divider column
- [ ] Lyrics edge padding -- small top/bottom margin so first/last lines are not flush against borders

### Defer (Future Milestones)

- [ ] Configurable layout -- if users request custom panel arrangement, treat as a separate milestone
- [ ] Smooth scroll interpolation for lyrics -- requires tracking fractional scroll position between frames

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Consistent spacing system | HIGH | MEDIUM | P1 |
| Artwork vertical centering | HIGH | LOW | P1 |
| Even button distribution | HIGH | LOW | P1 |
| Metadata clean alignment | HIGH | LOW | P1 |
| Lyrics current-line highlight | MEDIUM | LOW | P1 |
| No dead zones (fill + center) | HIGH | MEDIUM | P1 |
| Adaptive artwork/info split | MEDIUM | MEDIUM | P2 |
| Button text truncation | LOW | LOW | P2 |
| Separator line artwork/info | LOW | LOW | P2 |
| Lyrics edge padding | LOW | LOW | P2 |

**Priority key:**
- P1: Must complete for this milestone (addresses PROJECT.md active requirements)
- P2: Should complete if time permits (enhances polish beyond requirements)
- P3: Defer to future milestone

## Competitor Feature Analysis

| Feature | rmpc | spotify-player | ncmpcpp | termusic | AMCLI Current | AMCLI Target |
|---------|------|----------------|---------|----------|---------------|--------------|
| Album art display | Kitty/Sixel/iTerm2/ueberzug | Kitty/iTerm2/Sixel | External only | Kitty/iTerm2/Sixel | Kitty/Sixel/halfblocks | Same (no change) |
| Art centering | Centered in panel | Centered | N/A | Top-aligned | Top-left pinned | Vertically centered |
| Lyrics sync scroll | Yes, centered current line | Yes, synced | Yes | Yes | Yes, scroll to mid | Same + better highlight |
| Lyrics highlight | Bold + color | Bold | Reverse video | Bold + color | Bold only | Bold + accent color + graduated dim |
| Button/control layout | Statusbar-style | Statusbar | Keybind bar | Keybind hints | Bordered button row | Even-distributed bordered buttons |
| Spacing consistency | High (configurable theme) | High | High (mature) | Medium | Low (ad-hoc padding) | Systematic spacing constants |
| Adaptive layout | Configurable panels | Fixed but proportional | Configurable views | Fixed | Fixed percentages | Proportional with Min/Max |

## Ratatui 0.30 API Notes for Implementation

Key APIs available in Ratatui 0.30 that directly enable these features:

**Centering (new in 0.30):**
- `Rect::centered(h_constraint, v_constraint)` -- center both axes
- `Rect::centered_vertically(constraint)` -- vertical only
- `Rect::centered_horizontally(constraint)` -- horizontal only

**Flex distribution (stabilized in 0.30):**
- `Layout::horizontal([...]).flex(Flex::SpaceBetween)` -- even gaps between items
- `Layout::horizontal([...]).flex(Flex::SpaceEvenly)` -- even gaps including edges (new name in 0.30, was SpaceAround)
- `Layout::horizontal([...]).flex(Flex::Center)` -- center items as a group
- `.spacing(n)` -- fixed gap between items, works with any Flex mode

**Constraint types:**
- `Constraint::Fill(weight)` -- proportional fill of remaining space (use instead of Percentage for flexible areas)
- `Constraint::Min(n)` / `Constraint::Max(n)` -- bounded flexibility
- `Constraint::Length(n)` -- fixed size

**Layout ergonomics (new in 0.30):**
- `Rect::layout::<N>(&Layout)` -- compile-time checked split into `[Rect; N]`

The current codebase uses none of `Flex`, `Fill`, `centered()`, or the new `layout::<N>()` ergonomic methods. All of these are available in the project's declared dependency `ratatui = "0.30"`.

## Sources

- [Ratatui Layout Documentation](https://ratatui.rs/concepts/layout/)
- [Ratatui Center Widget Recipe](https://ratatui.rs/recipes/layout/center-a-widget/)
- [Ratatui Flex Example](https://ratatui.rs/examples/layout/flex/)
- [Ratatui 0.30 Highlights](https://ratatui.rs/highlights/v030/)
- [Ratatui Rect API Docs](https://docs.rs/ratatui/latest/ratatui/layout/struct.Rect.html)
- [Ratatui Constraint API Docs](https://docs.rs/ratatui/latest/ratatui/layout/enum.Constraint.html)
- [rmpc - Ratatui MPD Client](https://github.com/mierak/rmpc)
- [spotify-player - Rust Spotify TUI](https://github.com/aome510/spotify-player)
- [termusic - Rust Music Player TUI](https://github.com/tramhao/termusic)
- [ncmpcpp - ArchWiki](https://wiki.archlinux.org/title/Ncmpcpp)
- [LyricsMPRIS-Rust](https://github.com/BEST8OY/LyricsMPRIS-Rust)

---
*Feature research for: TUI music player layout and visual polish*
*Researched: 2026-03-26*
