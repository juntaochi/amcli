# AMCLI

## What This Is

A macOS terminal UI application for controlling Apple Music. Renders album artwork, synchronized lyrics, playback controls, and track metadata in a Ratatui-powered TUI. Communicates with Apple Music via AppleScript/osascript.

## Core Value

The TUI looks polished and adapts gracefully to any terminal size — artwork, lyrics, controls, and metadata all use available space well without breaking layout.

## Requirements

### Validated

- ✓ Play/pause, next, previous, volume, mute controls via AppleScript — existing
- ✓ Album artwork display with LRU caching and terminal protocol support (Sixel, Kitty, halfblocks) — existing
- ✓ Synchronized lyrics from multiple providers (local LRC files, Netease, LRCLIB) with priority fallback — existing
- ✓ Track metadata display (title, artist, album) — existing
- ✓ Progress bar with elapsed/total time — existing
- ✓ TOML-based config with language (en/ja), theme selection, artwork mode, mosaic effects — existing
- ✓ 6 color themes — existing
- ✓ Settings overlay menu — existing
- ✓ Non-blocking async operations (lyrics fetch, artwork download) via Tokio — existing
- ✓ Draw function decomposed into 8 isolated section renderers with narrow data slices — Phase 1

### Active

- [ ] Artwork vertically centered in its available area (not pinned to top-left)
- [ ] Control buttons evenly distributed across terminal width at any size
- [ ] Consistent spacing/margins between artwork, info, lyrics, and controls
- [ ] Song info area (title, artist, album) cleanly aligned and spaced
- [ ] Lyrics area uses available vertical space with better presentation and current-line highlight
- [ ] Layout fills available space proportionally rather than leaving dead zones

### Out of Scope

- Responsive collapse/hide at tiny sizes — user wants it to work at any size, not degrade
- Performance optimization — not a current concern
- New features (playlist, search, queue) — this milestone is layout/polish only

## Context

- The `draw()` function is now a 99-line orchestrator dispatching to 8 section renderers (draw_controls, draw_progress, draw_idle, draw_chassis, draw_screen_border, draw_artwork, draw_metadata, draw_lyrics). Safe to modify individual sections independently.
- `src/ui/mod.rs` contains all rendering logic plus App state. Section renderers receive narrow data slices (Frame, Rect, specific fields) instead of &mut App.
- Ratatui uses a `Rect`-based constraint layout system. Centering and proportional distribution are done via `Layout`, `Constraint`, and `Alignment` types.
- Terminal protocols for artwork (Sixel, Kitty, halfblocks) have different aspect ratio behaviors that affect vertical centering.
- The UI is rendered in Japanese (configurable to English). Labels like 曲名, アーティスト, アルバム, 再生, 次, 前, etc.

## Constraints

- **Platform**: macOS only — requires Apple Music app
- **Framework**: Ratatui 0.30 + Crossterm 0.28 — all layout via Ratatui constraint system
- **Terminal size**: Should look good at any reasonable terminal size, no minimum
- **No new dependencies**: Prefer using existing Ratatui layout primitives

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Center & fill strategy over graceful collapse | User wants the layout to always fill and center, not hide elements | — Pending |
| Layout + visual polish combined | Both layout fixes and spacing/alignment polish in same milestone | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-26 after Phase 1 completion*
