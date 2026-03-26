# Phase 1: Draw Function Decomposition - Research

**Researched:** 2026-03-26
**Domain:** Ratatui TUI draw function decomposition (Rust borrow semantics, Ratatui 0.30 layout APIs)
**Confidence:** HIGH

## Summary

The monolithic `draw()` function in `src/ui/mod.rs` (lines 617--1071, 454 lines) performs all rendering inline: chassis chrome, screen border, artwork, metadata, lyrics, progress bar, controls, and settings overlay. Decomposing it requires understanding the exact data flow into each section, Rust borrow checker constraints on `&mut App`, and the rendering order contract (settings overlay must be last).

The existing codebase already demonstrates the target pattern twice: `draw_lyrics()` (lines 567--615, a free function taking `Frame + Rect + &App`) and `settings_menu.render()` (in `src/ui/settings.rs`, a method taking `Frame + Theme`). The decomposition extends this pattern to the remaining six sections.

The primary risk is borrow checker conflicts when extracting artwork rendering, which requires `&mut app.throbber_state` and `&mut app.artwork_protocol`. The solution is destructuring these fields in the orchestrator before calling section renderers -- passing individual mutable references rather than `&mut App`.

**Primary recommendation:** Extract section renderers as free functions in `src/ui/mod.rs` (same file, not new files), passing narrow data slices. Keep the file structure flat for Phase 1 -- file reorganization into `render/` submodule is a future phase concern.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
None -- all implementation choices are at Claude's discretion (infrastructure phase).

### Claude's Discretion
All implementation choices are at Claude's discretion -- pure infrastructure phase. Use ROADMAP phase goal, success criteria, and codebase conventions to guide decisions.

Key guidance from research:
- Use free-function section renderers (not custom Widget structs) -- internal sections, not reusable
- Pass narrow data slices (Frame, Rect, theme, specific fields) -- not &mut App
- Existing draw_lyrics() and settings.render() serve as the pattern to follow
- Extract leaf sections first (controls, progress, idle), then complex ones (artwork, metadata)
- Application must render identically to before -- no visual changes

### Deferred Ideas (OUT OF SCOPE)
None -- infrastructure phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STRC-01 | Draw function decomposed into orchestrator + section renderers with narrow data slices | Draw function mapped line-by-line; 8 sections identified with exact data dependencies; orchestrator pattern documented with borrow-safe destructuring |
| STRC-02 | Each section renderer is a standalone function receiving Frame, Rect, and relevant state | Function signatures designed for all 8 sections; narrow parameter lists avoid &mut App; borrow checker conflicts for artwork/throbber resolved via field destructuring |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Framework:** Ratatui 0.30.0 + Crossterm 0.28 -- all layout via Ratatui constraint system
- **No new dependencies:** Prefer using existing Ratatui layout primitives
- **Never block the UI draw loop** with I/O
- **Commit format:** `<type>(<scope>): <subject>`
- **Error handling:** `anyhow::Result` for application logic
- **CI enforcement:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release` (all with `--all-features`)
- **Verify locally with:** `make verify`

## Current Draw Function: Exact Section Map

The `draw()` function (lines 617--1071) contains these sections in rendering order:

| # | Section | Lines | Render Order | Data Read | Mutable State |
|---|---------|-------|-------------|-----------|---------------|
| 1 | Background fill | 622 | First (base layer) | `theme.bg` | None |
| 2 | Chassis chrome | 624--671 | Second | `theme`, `is_jp` | None (returns `chassis_inner: Rect`) |
| 3 | Top-level layout split | 673--684 | N/A (computation) | `chassis_inner` | None (produces display_area, tuner_area, control_area) |
| 4 | Screen border (retro) | 686--697 | Third | `theme`, `display_area` | None (produces `screen_inner: Rect`) |
| 5 | Content layout split | 699--715 | N/A (computation) | `config.artwork.album`, `display_area.width` | None |
| 6 | Artwork | 717--772 | Fourth | `config`, `theme`, `is_jp` | `&mut throbber_state`, `&mut artwork_protocol` |
| 7 | Metadata/Info split | 774--803 | N/A (computation) | `current_lyrics.is_some()`, `info_chunk` dimensions | None |
| 8 | Metadata (track present) | 805--935 | Fifth | `current_track` (cloned), `theme`, `is_jp`, `animation_frame` | None |
| 9 | Idle (no track) | 936--963 | Fifth (alt) | `theme`, `is_jp` | None |
| 10 | Lyrics | 965--968 | Sixth | `current_track`, `current_lyrics`, `theme` | None (delegates to `draw_lyrics`) |
| 11 | Progress bar | 970--1003 | Seventh | `current_track`, `theme` | None |
| 12 | Controls | 1006--1065 | Eighth | `theme`, `is_jp`, `control_area` | None |
| 13 | Settings overlay | 1067--1070 | Last (topmost z-order) | `settings_menu.is_open` | None (delegates to `settings_menu.render`) |

### Key Observations

1. **Only artwork needs `&mut`:** `throbber_state` (line 751) and `artwork_protocol` (line 754) are the only mutable borrows during rendering. Every other section reads immutable data.

2. **Chassis returns a Rect:** The chassis section (lines 624--671) produces `chassis_inner` which all subsequent sections depend on. This section MUST either return the inner Rect or be folded into the orchestrator.

3. **Screen border also returns a Rect:** Lines 686--697 produce `screen_inner` from `display_area`. Same return-Rect pattern as chassis.

4. **Layout computation is interleaved:** Lines 673--684, 699--715, and 774--803 compute layout splits between render calls. These stay in the orchestrator.

5. **Metadata clones the track:** Line 805 does `app.get_current_track().cloned()`. This clone means the metadata section does NOT hold a borrow on `app` -- it works with an owned `Track`.

6. **draw_lyrics takes `&App`:** The existing `draw_lyrics` (line 567) takes `&App`, which works because it only reads immutable fields. But this means it cannot be called while `&mut app.throbber_state` is borrowed. The call on line 967 happens AFTER artwork rendering completes, so there is no current conflict, but the orchestrator must maintain this ordering.

7. **Settings overlay renders last -- this is a hard contract.** Ratatui has no z-index. Render order IS z-order.

## Borrow Checker Analysis

### The Problem

```rust
pub fn draw(f: &mut Frame, app: &mut App) {
    // ... orchestrator uses &mut app ...

    // ARTWORK: needs &mut app.throbber_state AND &mut app.artwork_protocol
    f.render_stateful_widget(loader, art_rect, &mut app.throbber_state);  // line 751
    f.render_stateful_widget(image, art_rect, protocol);                   // line 754

    // METADATA: needs app.get_current_track() (&app borrow)
    if let Some(track) = app.get_current_track().cloned() { ... }          // line 805

    // LYRICS: needs &app (immutable borrow)
    draw_lyrics(f, lyrics_area, app);                                       // line 967
}
```

Currently this all works because the compiler can see all field accesses in one scope. When we extract to sub-functions, passing `&mut App` to the artwork renderer would conflict with later `&App` borrows.

### The Solution: Field Destructuring in Orchestrator

```rust
pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = app.current_theme();           // Copy -- no borrow
    let is_jp = app.config.general.language == Language::Japanese;  // Copy

    // ... layout computation ...

    // Destructure mutable fields BEFORE calling renderers
    draw_artwork(
        f,
        art_rect,
        app.artwork_protocol.as_mut(),    // Option<&mut StatefulProtocol>
        app.is_loading_artwork,           // bool (Copy)
        &mut app.throbber_state,          // &mut ThrobberState
        theme,                            // Theme (Copy)
        is_jp,                            // bool (Copy)
    );

    // After artwork renderer returns, mutable borrows are released.
    // Now safe to take immutable borrows:

    draw_metadata(
        f,
        metadata_area,
        app.current_track.as_ref(),       // Option<&Track>
        app.animation_frame,              // u32 (Copy)
        theme,
        is_jp,
    );

    draw_lyrics_section(
        f,
        lyrics_area,
        app.current_track.as_ref(),
        app.current_lyrics.as_ref(),
        theme,
    );
}
```

This works because:
- Rust allows simultaneous borrows of DIFFERENT fields of a struct
- `theme` is `Copy`, so no borrow at all
- Mutable borrows (`&mut throbber_state`, `artwork_protocol.as_mut()`) are scoped to the `draw_artwork` call and released when it returns
- Subsequent immutable borrows (`current_track.as_ref()`, `current_lyrics.as_ref()`) do not conflict

### Fields That Are Copy (No Borrow Issues)

| Field | Type | Copy? |
|-------|------|-------|
| `current_theme()` | `Theme` | Yes -- derives `Copy` (line 35) |
| `is_loading_artwork` | `bool` | Yes |
| `animation_frame` | `u32` | Yes |
| `show_help` | `bool` | Yes |
| `volume` | `u8` | Yes |
| `is_muted` | `bool` | Yes |
| `config.general.language` | `Language` | Needs verification (likely Copy via derive) |
| `config.artwork.album` | `bool` | Yes |

### Fields That Need Borrowing

| Field | Type | Borrow Kind | Used By |
|-------|------|-------------|---------|
| `artwork_protocol` | `Option<StatefulProtocol>` | `&mut` (for `render_stateful_widget`) | Artwork |
| `throbber_state` | `ThrobberState` | `&mut` (for `render_stateful_widget`) | Artwork |
| `current_track` | `Option<Track>` | `&` (via `.as_ref()`) | Metadata, Progress, Lyrics |
| `current_lyrics` | `Option<Lyrics>` | `&` (via `.as_ref()`) | Lyrics |
| `settings_menu` | `SettingsMenu` | `&` (for `.render()`) | Settings overlay |
| `config` | `Config` | `&` (for `artwork.album`, `artwork.mode`) | Orchestrator layout decisions |

## Architecture Patterns

### Pattern: Free-Function Section Renderers

Each visual section becomes a standalone `fn` taking `&mut Frame`, `Rect`, and narrow data slices. NOT custom Widget structs (overkill for internal sections), NOT methods on App (couples rendering to state).

**Signature template:**
```rust
fn draw_section_name(
    f: &mut Frame,
    area: Rect,
    // ... only the data this section reads ...
    theme: Theme,
) { ... }
```

**Existing examples in the codebase:**
- `draw_lyrics(f: &mut Frame, area: Rect, app: &App)` -- lines 567--615 (takes `&App`, broader than ideal)
- `settings_menu.render(&self, f: &mut Frame, theme: Theme)` -- settings.rs line 174 (method on struct)

### Proposed Function Signatures

```rust
/// Chassis chrome: retro borders + scanlines, or passthrough for modern themes.
/// Returns the inner area where content should be rendered.
fn draw_chassis(f: &mut Frame, area: Rect, theme: Theme, is_jp: bool) -> Rect;

/// Screen border (retro double-border around display area).
/// Returns the inner area. Folded into chassis or kept separate.
fn draw_screen_border(f: &mut Frame, area: Rect, theme: Theme) -> Rect;

/// Artwork: loading spinner, image, or "NO SIGNAL" placeholder.
fn draw_artwork(
    f: &mut Frame,
    area: Rect,
    protocol: Option<&mut StatefulProtocol>,
    is_loading: bool,
    throbber_state: &mut ThrobberState,
    theme: Theme,
    is_jp: bool,
);

/// Track metadata: status line, labels, values with scroll text.
fn draw_metadata(
    f: &mut Frame,
    area: Rect,
    track: &Track,
    animation_frame: u32,
    is_two_columns: bool,
    theme: Theme,
    is_jp: bool,
);

/// Idle state: "WAITING FOR MEDIA INPUT..." when no track is playing.
fn draw_idle(f: &mut Frame, area: Rect, theme: Theme, is_jp: bool);

/// Lyrics: synced lyrics with current-line highlight.
fn draw_lyrics_section(
    f: &mut Frame,
    area: Rect,
    track: &Track,
    lyrics: &Lyrics,
    theme: Theme,
);

/// Progress bar gauge with time label.
fn draw_progress(f: &mut Frame, area: Rect, track: &Track, theme: Theme);

/// Control buttons row.
fn draw_controls(f: &mut Frame, area: Rect, theme: Theme, is_jp: bool);
```

### Orchestrator Structure

```rust
pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.current_theme();
    let is_jp = app.config.general.language == Language::Japanese;

    // 1. Background
    f.render_widget(Block::default().style(Style::default().bg(theme.bg)), area);

    // 2. Chassis + screen border (returns inner rects)
    let chassis_inner = draw_chassis(f, area, theme, is_jp);
    let [display_area, tuner_area, control_area] = Layout::vertical([
        Constraint::Min(10),
        Constraint::Length(3),
        Constraint::Length(3),
    ]).split(chassis_inner).to_vec().try_into().unwrap();
    // NOTE: or use chassis_inner.layout(&Layout::vertical(...)) if available

    let screen_inner = draw_screen_border(f, display_area, theme);

    // 3. Content layout (artwork + info split)
    let show_artwork = app.config.artwork.album && display_area.width > 50;
    // ... compute content_layout, info_chunk, metadata_area, lyrics_area ...

    // 4. Artwork (mutable borrows -- call FIRST, release before immutable borrows)
    if show_artwork {
        draw_artwork(
            f, artwork_area,
            app.artwork_protocol.as_mut(),
            app.is_loading_artwork,
            &mut app.throbber_state,
            theme, is_jp,
        );
    }

    // 5. Metadata or Idle (immutable borrows -- safe after artwork returns)
    if let Some(track) = app.current_track.as_ref() {
        draw_metadata(f, metadata_area, track, app.animation_frame, is_two_columns, theme, is_jp);
    } else {
        draw_idle(f, info_chunk, theme, is_jp);
    }

    // 6. Lyrics
    if lyrics_area.height > 2 {
        if let (Some(track), Some(lyrics)) =
            (app.current_track.as_ref(), app.current_lyrics.as_ref())
        {
            draw_lyrics_section(f, lyrics_area, track, lyrics, theme);
        }
    }

    // 7. Progress
    if let Some(track) = app.current_track.as_ref() {
        draw_progress(f, tuner_area, track, theme);
    }

    // 8. Controls
    draw_controls(f, control_area, theme, is_jp);

    // 9. Settings overlay -- MUST BE LAST (z-order contract)
    if app.settings_menu.is_open {
        app.settings_menu.render(f, theme);
    }
}
```

### Anti-Patterns to Avoid

- **Passing `&mut App` to every renderer:** Creates invisible coupling and borrow conflicts. Pass narrow data slices.
- **Custom Widget structs:** `struct ArtworkWidget { ... } impl Widget for ArtworkWidget` adds ceremony with no benefit for internal-only sections.
- **Layout computation inside section renderers:** Each renderer receives its pre-computed `Rect`. Inter-section layout stays in the orchestrator.
- **Moving files prematurely:** Do NOT create a `render/` submodule in Phase 1. Extract functions within `src/ui/mod.rs` first. File reorganization is a separate concern.

## Extraction Order (Safe Build Sequence)

Extract in this order to maintain compilability at each step:

| Step | What | Why This Order | Risk |
|------|------|----------------|------|
| 1 | `draw_controls()` | Self-contained leaf. No return value. Only reads `theme` and `is_jp`. | LOW -- purely mechanical extraction |
| 2 | `draw_progress()` | Self-contained leaf. Only reads `&Track` and `theme`. | LOW |
| 3 | `draw_idle()` | Self-contained leaf. Only reads `theme` and `is_jp`. | LOW |
| 4 | `draw_chassis()` | Returns `Rect`. No mutable state. The retro/modern branch is cleanly bounded. | LOW -- must verify returned Rect matches original |
| 5 | `draw_screen_border()` | Returns `Rect`. Clean extraction from lines 686--697. Can be folded into chassis. | LOW |
| 6 | `draw_artwork()` | The borrow-sensitive section. Needs `&mut throbber_state`, `&mut artwork_protocol`. | MEDIUM -- requires orchestrator field destructuring |
| 7 | `draw_metadata()` | Complex internal logic (two-column vs single-column). Currently clones Track. After extraction, pass `&Track` instead. | MEDIUM -- eliminate the `.cloned()` on line 805 |
| 8 | Refactor `draw_lyrics()` | Already extracted. Narrow its signature from `&App` to individual fields. | LOW -- signature change only |
| 9 | Wire orchestrator | Replace inline code in `draw()` with calls to extracted functions. | MEDIUM -- must maintain render order and layout computation |

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Vertical centering | Manual 3-part split (current lines 728--735, 762--769) | `Rect::centered_vertically(Constraint::Length(n))` | Ratatui 0.30 built-in, one line vs six |
| Horizontal centering | Manual 3-part split (current lines 737--744) | `Rect::centered_horizontally(Constraint::Length(n))` | Same |
| Layout splitting | `Layout::default().direction().constraints().split()` (verbose) | `area.layout(&Layout::vertical([...]))` returning `[Rect; N]` | Ratatui 0.30 ergonomic API, const-generic array return |
| Rect construction | `Rect::new(x, y, w, h)` for centered popups | `Rect::centered(h_constraint, v_constraint)` | Less arithmetic, less error-prone |

**Note:** The centering APIs and `Rect::layout()` are available in Ratatui 0.30.0 (confirmed in Cargo.lock). However, Phase 1 should NOT change layout logic -- only extract it. Using these new APIs is a Phase 2 concern. Phase 1 copies the existing layout code verbatim into the extracted functions.

## Common Pitfalls

### Pitfall 1: Borrow Conflict on Artwork Extraction
**What goes wrong:** Extracting artwork rendering into a function that takes `&mut App` prevents subsequent `&App` borrows for metadata/lyrics.
**Why it happens:** Rust cannot do partial borrows through function signatures.
**How to avoid:** Pass individual fields: `app.artwork_protocol.as_mut()`, `&mut app.throbber_state`, `app.is_loading_artwork`. See "Borrow Checker Analysis" section above.
**Warning signs:** Compiler error E0499 or E0502 after extraction.

### Pitfall 2: Breaking Settings Overlay Z-Order
**What goes wrong:** Settings overlay rendered before other widgets becomes invisible.
**Why it happens:** Ratatui has no z-index; render order IS z-order.
**How to avoid:** Settings overlay call MUST be the last render call in the orchestrator. Add a comment: `// LAST: Settings overlay (z-order contract)`.
**Warning signs:** Settings menu not visible when opened after refactoring.

### Pitfall 3: Retro Theme Chassis-to-Screen Gap
**What goes wrong:** After extracting `draw_chassis()`, the returned `chassis_inner` Rect does not match what `draw_screen_border()` expects, producing a 1-cell gap between the chassis border and the screen border.
**Why it happens:** `chassis_block.inner(area)` computes padding from the Block's borders. If the Block construction changes even slightly during extraction (different border type, missing border side), the inner Rect shifts.
**How to avoid:** Copy the chassis Block construction EXACTLY from the original code. Verify by running with a retro theme before and after extraction and comparing visually.
**Warning signs:** Visual gap between chassis and screen borders in retro themes. Clean theme unaffected (passes through area unchanged).

### Pitfall 4: Metadata Track Clone Becoming Double-Clone
**What goes wrong:** The current code does `app.get_current_track().cloned()` on line 805, cloning the entire Track. If the extracted `draw_metadata()` function takes `&Track`, the orchestrator passes `app.current_track.as_ref()` -- no clone needed. But if someone accidentally keeps the `.cloned()` AND passes the owned Track, it is wasted allocation every frame.
**How to avoid:** The extracted `draw_metadata()` takes `&Track`. The orchestrator passes `app.current_track.as_ref().unwrap()` (guarded by an `if let Some`). Remove the `.cloned()` call entirely.
**Warning signs:** Unnecessary `Track` clone visible in the orchestrator code.

### Pitfall 5: draw_lyrics Signature Narrowing Breaks Existing Tests
**What goes wrong:** The existing `draw_lyrics()` (line 567) takes `&App`. Changing its signature to take individual fields may break the test on line 1204 which calls `draw(f, &mut app)` (and `draw` calls `draw_lyrics`).
**How to avoid:** The test calls `draw()`, not `draw_lyrics()` directly. Changing `draw_lyrics`'s signature is safe as long as the orchestrator adapts its call site. No external callers exist.
**Warning signs:** Test compilation failure after signature change.

## Code Examples

### Example 1: Extracting draw_controls (simplest case)

```rust
// BEFORE (inline in draw(), lines 1006-1065):
let controls = if is_jp { vec![...] } else { vec![...] };
let btn_width = control_area.width / controls.len() as u16;
// ... 30 lines of button rendering ...

// AFTER (extracted function):
fn draw_controls(f: &mut Frame, area: Rect, theme: Theme, is_jp: bool) {
    let controls = if is_jp {
        vec![
            ("▶再生", "SPC"), ("▶▶次", "]"), ("◀◀前", "["),
            ("音量＋", "+"), ("音量－", "-"), ("消音", "m"), ("電源", "q"),
        ]
    } else {
        vec![
            ("PLAY", "SPC"), ("SKIP", "]"), ("PREV", "["),
            ("VOL+", "+"), ("VOL-", "-"), ("MUTE", "m"), ("EXIT", "q"),
        ]
    };

    let btn_width = area.width / controls.len() as u16;
    let btn_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(btn_width); controls.len()])
        .split(area);

    for (i, (label, key)) in controls.iter().enumerate() {
        if i < btn_layout.len() {
            // ... exact same button rendering code ...
        }
    }
}

// Orchestrator call:
draw_controls(f, control_area, theme, is_jp);
```

### Example 2: Extracting draw_artwork (borrow-sensitive case)

```rust
fn draw_artwork(
    f: &mut Frame,
    area: Rect,
    protocol: Option<&mut StatefulProtocol>,
    is_loading: bool,
    throbber_state: &mut ThrobberState,
    theme: Theme,
    is_jp: bool,
) {
    let h_padding = 2;
    let side = area.width.saturating_sub(h_padding * 2);
    let char_height = side / 2;
    let v_padding = (area.height.saturating_sub(char_height)) / 2;

    // Existing layout code (copied verbatim from lines 728-744)
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

    if is_loading {
        let loader = Throbber::default()
            .throbber_set(BRAILLE_SIX_DOUBLE)
            .use_type(WhichUse::Spin)
            .style(Style::default().fg(theme.accent));
        f.render_stateful_widget(loader, art_rect, throbber_state);
    } else if let Some(protocol) = protocol {
        let image = StatefulImage::default();
        f.render_stateful_widget(image, art_rect, protocol);
    } else {
        let no_sig_text = if is_jp { "信号なし" } else { "NO SIGNAL" };
        let no_sig = Paragraph::new(no_sig_text)
            .style(Style::default().fg(theme.dim).add_modifier(Modifier::DIM))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        let v_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(art_rect);
        f.render_widget(no_sig, v_center[1]);
    }
}

// Orchestrator call:
if show_artwork {
    draw_artwork(
        f,
        artwork_column,
        app.artwork_protocol.as_mut(),
        app.is_loading_artwork,
        &mut app.throbber_state,
        theme,
        is_jp,
    );
}
```

### Example 3: Chassis extraction returning Rect

```rust
fn draw_chassis(f: &mut Frame, area: Rect, theme: Theme, is_jp: bool) -> Rect {
    if theme.is_retro {
        let chassis_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(theme.dim))
            .title(vec![/* ... exact same title spans ... */])
            .title_alignment(Alignment::Center)
            .title_bottom(vec![/* ... exact same bottom title spans ... */])
            .title_alignment(Alignment::Center);

        let inner = chassis_block.inner(area);
        f.render_widget(chassis_block, area);

        // Scanlines
        for y in (inner.top()..inner.bottom()).step_by(2) {
            let line = Paragraph::new(" ".repeat(inner.width as usize))
                .style(Style::default().bg(Color::Rgb(5, 5, 5)).add_modifier(Modifier::DIM));
            f.render_widget(line, Rect::new(inner.left(), y, inner.width, 1));
        }
        inner
    } else {
        area
    }
}
```

## Alignment Migration (Preparatory Step)

Ratatui 0.30.0 renamed `Alignment` to `HorizontalAlignment` (PR #1735). A type alias preserves backward compatibility, so the current code compiles fine. However:

- The existing code uses `Alignment::Center` in 8 places across `src/ui/mod.rs` and 3 places in `src/ui/settings.rs`
- The type alias works but the canonical name is now `HorizontalAlignment`
- Phase 1 should NOT migrate this. It is a mechanical rename that can be done in a separate commit before or after decomposition. Mixing structural refactoring with name changes makes diffs harder to review.

**Recommendation:** Do the `Alignment` -> `HorizontalAlignment` rename as a standalone preparatory commit BEFORE function extraction. This keeps each commit focused on one kind of change.

## Import Changes Required

The current `src/ui/mod.rs` imports (line 4):
```rust
layout::{Alignment, Constraint, Direction, Layout},
```

After extraction, `Rect` must be added to the import since extracted functions use it in their signatures:
```rust
layout::{Alignment, Constraint, Direction, Layout, Rect},
```

Currently `Rect` is referenced via full path `ratatui::layout::Rect` in three places (lines 567, 665, 802). The extraction should import it at the top and use the short name.

## Existing Test Coverage

The test on lines 1195--1211 (`test_ui_rendering`) calls `draw(f, &mut app)` and checks that the buffer contains "TEST", "SONG", and "ARTIST" strings. This test:

- **Validates the orchestrator works end-to-end** -- it will continue to pass as long as the extracted functions produce identical output
- **Does NOT test individual section renderers** -- no isolated tests for controls, progress, etc.
- **Uses `TestBackend::new(120, 40)`** -- a generous terminal size that does not exercise small-size edge cases

After decomposition, the existing test serves as a regression guard. No new tests are required for Phase 1 (the goal is identical output), but individual section renderers become independently testable for future phases.

## Open Questions

1. **Should `draw_chassis` and `draw_screen_border` be one function or two?**
   - Current code has them as separate blocks (lines 624--671 and 686--697)
   - They are both theme-conditional and both return Rects
   - Recommendation: Keep them separate. `draw_chassis` operates on the full terminal area; `draw_screen_border` operates on `display_area` (a sub-rect). Different inputs, different outputs.
   - Confidence: MEDIUM -- either approach works; separation is slightly cleaner.

2. **Should `draw_lyrics` signature be narrowed in Phase 1 or deferred?**
   - Current signature: `draw_lyrics(f: &mut Frame, area: Rect, app: &App)`
   - Ideal signature: `draw_lyrics_section(f: &mut Frame, area: Rect, track: &Track, lyrics: &Lyrics, theme: Theme)`
   - Recommendation: Narrow it in Phase 1. The function is already extracted; changing the signature is low-risk and aligns with the STRC-02 requirement ("standalone function receiving Frame, Rect, and relevant state").
   - Confidence: HIGH -- mechanical change, no behavioral risk.

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection: `src/ui/mod.rs` (1212 lines), `src/ui/settings.rs` (335 lines), `src/main.rs` (145 lines)
- [Ratatui 0.30.0 Rect docs](https://docs.rs/ratatui/0.30.0/ratatui/layout/struct.Rect.html) -- `centered_vertically()`, `centered_horizontally()`, `layout()` method signatures verified
- [Ratatui 0.30 Highlights](https://ratatui.rs/highlights/v030/) -- `Alignment` rename, new Rect methods
- [Ratatui PR #1735](https://github.com/ratatui/ratatui/pull/1735) -- `Alignment` to `HorizontalAlignment` rename details
- [Ratatui Center Widget Recipe](https://ratatui.rs/recipes/layout/center-a-widget/) -- centering pattern confirmation

### Secondary (MEDIUM confidence)
- `.planning/research/ARCHITECTURE.md` -- decomposition order and pattern recommendations
- `.planning/research/PITFALLS.md` -- borrow checker, z-order, and retro theme pitfalls
- `.planning/codebase/CONCERNS.md` -- tech debt analysis of monolithic draw()

## Metadata

**Confidence breakdown:**
- Borrow analysis: HIGH -- verified by tracing every field access in draw() line by line
- Extraction order: HIGH -- dependency analysis from direct code inspection
- Function signatures: HIGH -- designed from actual data flow, not hypothetical
- Ratatui 0.30 APIs: HIGH -- verified against docs.rs and release notes
- Retro theme integrity: MEDIUM -- visual verification needed after extraction; code analysis alone cannot confirm pixel-perfect match

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable domain -- Ratatui 0.30 is the current version, Rust borrow semantics are permanent)
