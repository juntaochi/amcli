# Architecture Research

**Domain:** Ratatui TUI draw function decomposition and responsive layout
**Researched:** 2026-03-26
**Confidence:** HIGH

## System Overview

The current `draw()` function (455 lines) in `src/ui/mod.rs` is a single procedural function that computes layout, builds widgets, and renders everything inline. The recommended architecture decomposes this into a layout orchestrator calling discrete render functions, one per visual section.

```
┌──────────────────────────────────────────────────────────────────┐
│                      draw() — Layout Orchestrator                │
│  Computes top-level Rects, delegates to section renderers        │
├──────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌──────────────┐   │
│  │ Chassis  │  │  Artwork  │  │ Metadata │  │   Lyrics     │   │
│  │ + Screen │  │           │  │          │  │              │   │
│  └────┬─────┘  └─────┬─────┘  └────┬─────┘  └──────┬───────┘   │
│       │              │             │               │            │
│  ┌────┴─────┐  ┌─────┴─────┐  ┌───┴────┐  ┌──────┴───────┐   │
│  │ Progress │  │ Controls  │  │Settings│  │   Idle       │   │
│  │   Bar    │  │  Buttons  │  │Overlay │  │  (no track)  │   │
│  └──────────┘  └───────────┘  └────────┘  └──────────────┘   │
├──────────────────────────────────────────────────────────────────┤
│                  Shared: Theme, Language, App State               │
└──────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Current Location (lines) |
|-----------|----------------|--------------------------|
| `draw()` orchestrator | Top-level layout split, area assignment | 617-684 |
| Chassis + Screen | Retro border chrome, scanlines, screen border | 624-697 |
| Artwork | Vertical/horizontal centering, loader, image, no-signal | 717-772 |
| Metadata | Track info labels+values, single/two-column layout, scroll text | 805-963 |
| Lyrics | Synced lyrics with current-line highlight, scroll position | 567-615 (already extracted) |
| Progress bar | Gauge with time label | 970-1003 |
| Controls | Button row with even distribution | 1006-1065 |
| Settings overlay | Modal popup (already in `settings.rs`) | 1067-1070 (delegates) |
| Idle state | "Waiting for media" when no track | 936-963 |

## Recommended Project Structure

```
src/ui/
├── mod.rs              # Re-exports, App struct definition
├── app.rs              # App methods (update, event handling, state accessors)
├── theme.rs            # Theme struct, THEMES array, color constants
├── render/
│   ├── mod.rs          # draw() orchestrator — layout computation + delegation
│   ├── chassis.rs      # draw_chassis() — retro borders, scanlines, screen frame
│   ├── artwork.rs      # draw_artwork() — centering, loader, image, no-signal
│   ├── metadata.rs     # draw_metadata() — track info, single/two-col, scroll
│   ├── lyrics.rs       # draw_lyrics() — synced lyrics with highlight
│   ├── progress.rs     # draw_progress() — gauge with time label
│   ├── controls.rs     # draw_controls() — button row
│   └── idle.rs         # draw_idle() — no-track waiting state
├── settings.rs         # Existing settings overlay (already separated)
└── helpers.rs          # format_duration, scroll_text, format_duration_seconds
```

### Structure Rationale

- **`render/` sub-module:** Groups all rendering code separately from state management and event handling. Each file corresponds to a visual section of the UI. The `mod.rs` in `render/` is the orchestrator that replaces the current monolithic `draw()`.
- **`app.rs` separated from `mod.rs`:** The `App` struct and its methods (`update()`, `handle_key()`, etc.) are state management, not rendering. Separating them from the render tree prevents the current 1212-line file problem.
- **`theme.rs` standalone:** Themes are constants used by every render function. Extracting them to their own file avoids every render file importing from `mod.rs`.
- **`helpers.rs`:** Pure formatting functions (`format_duration`, `scroll_text`) are shared utilities, not tied to any single render section.

## Architectural Patterns

### Pattern 1: Free-Function Section Renderers

**What:** Each visual section becomes a standalone function taking `Frame`, `Rect`, and a narrow slice of `App` state. No custom Widget trait implementations needed.

**When to use:** When refactoring an existing monolithic draw function where sections do not need independent reuse outside this app. This is the right pattern for AMCLI because the sections (chassis, metadata, controls) are specific to this app's layout and not reusable library widgets.

**Trade-offs:** Simpler than the `impl Widget for &MyStruct` pattern. No new types needed. Each function is independently testable with `TestBackend`. The downside is less encapsulation than proper widgets, but for an internal app with 7 sections, the overhead of creating widget structs is not justified.

**Example:**

```rust
// src/ui/render/mod.rs

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.current_theme();
    let is_jp = app.is_japanese();

    // Background fill
    f.render_widget(Block::default().style(Style::default().bg(theme.bg)), area);

    // Chassis chrome (retro border + scanlines, or passthrough for modern)
    let screen_inner = chassis::draw_chassis(f, area, theme, is_jp);

    // Top-level vertical split: display | progress | controls
    let [display_area, tuner_area, control_area] = screen_inner.layout(
        &Layout::vertical([
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
    );

    // Content area — artwork left, info right (or full-width if no artwork)
    let show_artwork = app.config.artwork.album && display_area.width > 50;
    draw_content(f, app, display_area, show_artwork, theme, is_jp);

    // Progress bar
    if let Some(track) = app.get_current_track() {
        progress::draw_progress(f, tuner_area, track, theme);
    }

    // Controls
    controls::draw_controls(f, control_area, theme, is_jp);

    // Settings overlay (last — renders on top)
    if app.settings_menu.is_open {
        app.settings_menu.render(f, theme);
    }
}
```

### Pattern 2: Narrow Data Slices (Borrow Narrowing)

**What:** Render functions accept only the specific data they need, not `&App` or `&mut App`. This avoids borrow conflicts and makes dependencies explicit.

**When to use:** When a render function only needs 2-3 fields from App. Particularly important for the artwork renderer (needs `artwork_protocol`, `is_loading_artwork`, `throbber_state`) and the metadata renderer (needs `current_track`, `animation_frame`).

**Trade-offs:** More parameters per function, but eliminates hidden dependencies and makes each function independently testable without constructing a full App. The one exception is `draw_lyrics`, which needs `&App` because it accesses `current_lyrics`, `current_track`, and `theme` together -- acceptable since it is already extracted.

**Example:**

```rust
// src/ui/render/artwork.rs

pub fn draw_artwork(
    f: &mut Frame,
    area: Rect,
    protocol: Option<&mut StatefulProtocol>,
    is_loading: bool,
    throbber_state: &mut ThrobberState,
    theme: Theme,
    is_jp: bool,
) {
    // Centering computation using Ratatui 0.30 Rect methods
    let h_padding = 2u16;
    let side = area.width.saturating_sub(h_padding * 2);
    let char_height = side / 2;

    let art_rect = area
        .centered_vertically(Constraint::Length(char_height))
        .centered_horizontally(Constraint::Length(side));

    if is_loading {
        let loader = Throbber::default()
            .throbber_set(BRAILLE_SIX_DOUBLE)
            .use_type(WhichUse::Spin)
            .style(Style::default().fg(theme.accent));
        f.render_stateful_widget(loader, art_rect, throbber_state);
    } else if let Some(protocol) = protocol {
        f.render_stateful_widget(StatefulImage::default(), art_rect, protocol);
    } else {
        let text = if is_jp { "信号なし" } else { "NO SIGNAL" };
        let no_sig = Paragraph::new(text)
            .style(Style::default().fg(theme.dim).add_modifier(Modifier::DIM))
            .alignment(Alignment::Center);
        let centered = art_rect.centered_vertically(Constraint::Length(1));
        f.render_widget(no_sig, centered);
    }
}
```

### Pattern 3: Ratatui 0.30 Ergonomic Layout APIs

**What:** Use `Rect::layout()` (returns fixed-size array), `Rect::centered()` / `centered_vertically()` / `centered_horizontally()`, and `Layout::spacing()` instead of the verbose `Layout::default().direction().constraints().split()` pattern.

**When to use:** Everywhere. The current code uses the verbose 0.26-era pattern throughout. The 0.30 APIs reduce boilerplate and improve readability.

**Trade-offs:** Requires minor adjustment to constraint patterns. `Rect::layout()` needs the const generic count known at compile time (use `layout_vec()` for dynamic counts). Centering helpers eliminate the manual "split into 3 parts, take the middle" pattern used in at least 4 places in the current code.

**Example (before vs after):**

```rust
// BEFORE (current code, lines 728-744)
let art_rect = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(v_padding),
        Constraint::Length(char_height),
        Constraint::Min(0),
    ])
    .split(artwork_column)[1];

let art_rect = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(h_padding),
        Constraint::Length(side),
        Constraint::Min(0),
    ])
    .split(art_rect)[1];

// AFTER (Ratatui 0.30)
let art_rect = artwork_column
    .centered_vertically(Constraint::Length(char_height))
    .centered_horizontally(Constraint::Length(side));
```

## Data Flow

### Render Data Flow

```
App state (owned by main loop)
    │
    ├─ &mut App ──→ draw() orchestrator
    │                   │
    │                   ├─ theme: Theme (Copy)
    │                   ├─ is_jp: bool
    │                   │
    │                   ├─→ draw_chassis(f, area, theme, is_jp) → Rect (screen_inner)
    │                   │
    │                   ├─→ draw_artwork(f, rect, &mut protocol, loading, &mut throbber, theme, is_jp)
    │                   │
    │                   ├─→ draw_metadata(f, rect, &track, animation_frame, theme, is_jp)
    │                   │       └── calls scroll_text() for overflow
    │                   │
    │                   ├─→ draw_lyrics(f, rect, &lyrics, &track, theme)
    │                   │
    │                   ├─→ draw_progress(f, rect, &track, theme)
    │                   │
    │                   ├─→ draw_controls(f, rect, theme, is_jp)
    │                   │
    │                   └─→ settings_menu.render(f, theme)  [existing, unchanged]
    │
    └─ Layout decisions (show_artwork, is_two_columns) computed in orchestrator
```

### Key Data Flow Principles

1. **Layout decisions stay in the orchestrator.** Whether to show artwork, whether to use two-column metadata, and how to split content area -- these are inter-section dependencies that belong in the orchestrator, not in individual section renderers.

2. **Section renderers receive pre-computed Rects.** Each render function receives the exact area it should fill. It does not compute its own position relative to siblings. This keeps sections independent.

3. **Mutable borrows are narrow.** Only `draw_artwork` needs `&mut` (for `StatefulProtocol` and `ThrobberState`). All other section renderers take `&` references. The orchestrator destructures `app` fields before calling renderers to avoid conflicting borrows on `&mut App`.

4. **Theme is `Copy`.** The existing `Theme` struct derives `Copy`, so it passes freely to all renderers without borrow issues. This is already correct in the current code.

## Decomposition Order (Build Sequence)

The refactoring has dependency ordering that determines which sections can be extracted first without breaking compilation.

### Phase 1: Extract leaf sections (no inter-section dependencies)

These sections receive a `Rect` and data, render into it, and have no effect on other sections' layout.

| Order | Section | Why first | Depends on |
|-------|---------|-----------|------------|
| 1 | `helpers.rs` | Pure functions, zero render logic | Nothing |
| 2 | `theme.rs` | Constants, zero render logic | Nothing |
| 3 | `draw_controls()` | Self-contained, no layout effect on siblings | Theme, is_jp |
| 4 | `draw_progress()` | Self-contained, reads only Track | Theme, Track |
| 5 | `draw_idle()` | Self-contained, shown when no track | Theme, is_jp |

### Phase 2: Extract sections with internal layout complexity

These compute their own sub-layout within their Rect but do not affect other sections.

| Order | Section | Complexity | Depends on |
|-------|---------|-----------|------------|
| 6 | `draw_artwork()` | Centering logic, three render modes (loading/image/no-signal) | Theme, StatefulProtocol, ThrobberState |
| 7 | `draw_metadata()` | Single/two-column conditional layout, scroll_text | Theme, Track, animation_frame, is_jp |
| 8 | `draw_lyrics()` | Already extracted as a function, move to render/ module | Theme, Lyrics, Track |

### Phase 3: Extract the chassis and orchestrator

The chassis determines screen_inner which all other sections render into. The orchestrator computes the top-level layout splits.

| Order | Section | Why last | Depends on |
|-------|---------|----------|------------|
| 9 | `draw_chassis()` | Returns modified Rect that all content sections use | Theme, is_jp |
| 10 | `draw()` orchestrator | Calls all section renderers, computes layout splits | Everything |

### Phase 4: File structure refactoring

Move App struct methods and theme definitions out of `mod.rs`.

| Order | Action | Why after render extraction |
|-------|--------|---------------------------|
| 11 | Move Theme to `theme.rs` | Render functions already import from new locations |
| 12 | Move App to `app.rs` | All render functions are now in render/ and import narrowly |
| 13 | Slim `mod.rs` to re-exports | Final cleanup |

## Anti-Patterns

### Anti-Pattern 1: Custom Widget Structs for Internal Sections

**What people do:** Create `struct ArtworkWidget { ... }` with `impl Widget for ArtworkWidget` for each section, following the Ratatui custom widget pattern from the docs.

**Why it is wrong for this case:** Custom widget types add a struct definition + constructor + Widget impl per section. For an internal app where these "widgets" are never reused outside this one draw function, the type ceremony adds complexity without benefit. The sections need mutable access to App state (artwork_protocol, throbber_state) which conflicts with Widget's `self`-consuming signature -- you would need `impl Widget for &mut ArtworkWidget` with lifetime gymnastics.

**Do this instead:** Use free functions. `draw_artwork(f, area, ...)` is simpler, directly testable, and avoids Widget trait overhead. Reserve `impl Widget` for genuinely reusable components.

### Anti-Pattern 2: Passing `&mut App` to Every Render Function

**What people do:** Give each section renderer `fn draw_controls(f: &mut Frame, area: Rect, app: &mut App)` because it is easy.

**Why it is wrong:** Creates invisible coupling. Any section could read or write any App field. Makes it impossible to reason about what data each section actually needs. Also causes borrow checker fights when the orchestrator needs to pass different `&mut` slices to concurrent render calls.

**Do this instead:** Pass narrow data slices. `draw_controls(f, area, theme, is_jp)` makes the dependency graph explicit. When a section needs mutable state (artwork renderer needs `&mut ThrobberState`), destructure the relevant fields in the orchestrator before calling.

### Anti-Pattern 3: Layout Computation Inside Section Renderers

**What people do:** Have `draw_metadata()` compute where artwork goes, or `draw_artwork()` compute the info panel width, because the sections need to coordinate sizes.

**Why it is wrong:** Circular dependencies between section sizes. The artwork width affects the metadata width and vice versa. If each section computes its own layout relative to siblings, changes in one section break others.

**Do this instead:** All inter-section layout computation stays in the orchestrator. Section renderers receive their final Rect and fill it. The `show_artwork` and `is_two_columns` decisions live in the orchestrator.

### Anti-Pattern 4: Recreating Layout Objects Every Frame Without Caching

**What people do:** Call `Layout::default().direction().constraints().split()` with identical parameters every frame, which runs the Cassowary solver each time.

**Why it is wrong for hot paths:** The controls layout, chassis split, and content split are identical every frame unless the terminal resizes. For a 50ms frame rate, this is ~20 solver runs per second for static layouts.

**Do this instead:** For this app's frame rate (50ms poll, not a game), the overhead is negligible. But if performance matters later, `Layout` implements caching internally (it stores results keyed by area + constraints). The solver cost is low for 3-4 constraints. Do not prematurely optimize this -- focus on code clarity first.

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Orchestrator to section renderers | Function call with explicit parameters | One-way: orchestrator calls renderers, renderers do not call back |
| Section renderers to helpers | Function call (scroll_text, format_duration) | Pure functions, no state |
| Orchestrator to App state | Destructured field borrows from `&mut App` | Orchestrator is the only place that touches App during render |
| Settings overlay | `settings_menu.render(f, theme)` | Already separated, rendered last (on top) |

### Ratatui 0.30 API Surface Used

| API | Where Used | Replaces |
|-----|-----------|----------|
| `Rect::layout::<N>(&Layout) -> [Rect; N]` | Orchestrator, all fixed splits | `Layout::split()[index]` |
| `Rect::centered(h, v)` | Artwork centering, no-signal text | Manual 3-part split pattern |
| `Rect::centered_vertically(c)` | Lyrics "no lyrics" text, artwork | Manual vertical centering |
| `Layout::horizontal(constraints)` | Controls, two-column metadata | `Layout::default().direction(Horizontal)` |
| `Layout::vertical(constraints)` | Orchestrator main split | `Layout::default().direction(Vertical)` |
| `Layout::spacing(n)` | Controls button spacing | Manual Constraint::Length gaps |

## Sources

- [Ratatui Widget Concepts](https://ratatui.rs/concepts/widgets/) -- Widget, StatefulWidget, WidgetRef trait documentation (HIGH confidence)
- [Ratatui Layout Concepts](https://ratatui.rs/concepts/layout/) -- Constraint types, nested layouts, spacing (HIGH confidence)
- [Ratatui 0.30 Highlights](https://ratatui.rs/highlights/v030/) -- Rect::layout(), centered(), WidgetRef changes (HIGH confidence)
- [Ratatui Custom Widget Recipe](https://ratatui.rs/recipes/widgets/custom/) -- Widget implementation patterns (HIGH confidence)
- [Ratatui Component Architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/) -- Component trait pattern for large apps (HIGH confidence)
- [Ratatui Best Practices Discussion #220](https://github.com/ratatui/ratatui/discussions/220) -- MVC pattern, state management, draw decomposition (MEDIUM confidence)
- [Ratatui Center Widget Recipe](https://ratatui.rs/recipes/layout/center-a-widget/) -- Rect::centered() usage (HIGH confidence)
- [Ratatui Grid Layout Recipe](https://ratatui.rs/recipes/layout/grid/) -- Grid patterns, layout_vec() (HIGH confidence)
- [Ratatui Rendering Concepts](https://ratatui.rs/concepts/rendering/) -- Immediate mode, frame model (HIGH confidence)
- [Rect API docs](https://docs.rs/ratatui/latest/ratatui/layout/struct.Rect.html) -- Full method listing for Rect (HIGH confidence)
- Current codebase: `src/ui/mod.rs` lines 617-1071, `src/ui/settings.rs` (direct inspection)

---
*Architecture research for: Ratatui TUI draw function decomposition*
*Researched: 2026-03-26*
