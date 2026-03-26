---
phase: 03-section-level-polish
plan: 02
subsystem: ui
tags: [ratatui, layout, fill-constraints, lyrics, dimming, graduated-highlight]

# Dependency graph
requires:
  - phase: 03-section-level-polish
    provides: Rect::centered() artwork centering, Fill(1) button distribution (Plan 01)
  - phase: 02-layout-foundation
    provides: Fill-based proportional layout, SPACING constants, Layout::horizontal pattern
provides:
  - Fill(1)+Fill(1) two-column metadata layout eliminating Percentage rounding gap
  - 3-tier graduated lyrics dimming (accent+bold / primary / dim)
affects: [ui-rendering, visual-polish]

# Tech tracking
tech-stack:
  added: []
  patterns: [Fill(1) for equal-width columns, isize cast + unsigned_abs for safe distance, 3-tier style selection by distance]

key-files:
  created: []
  modified: [src/ui/mod.rs]

key-decisions:
  - "Fill(1)+Fill(1) with spacing(SPACING_NORMAL) for two-column metadata instead of Percentage(50)+Percentage(50)"
  - "theme.accent + BOLD for current lyrics line (brighter/warmer than primary in all themes)"
  - "3-tier distance thresholds: 0 = accent+bold, 1-2 = primary, 3+ = dim"
  - "isize cast before subtraction + unsigned_abs for safe unsigned distance calculation"

patterns-established:
  - "Fill(1)+Fill(1): use for equal-width column splits instead of Percentage(50) to avoid rounding gaps"
  - "Graduated dimming: calculate distance from focus index, select from tier array of styles"

requirements-completed: [VISL-02, VISL-03, VISL-04]

# Metrics
duration: 3min
completed: 2026-03-26
---

# Phase 3 Plan 2: Metadata Alignment and Lyrics Dimming Summary

**Fill(1) two-column metadata eliminating Percentage rounding gaps, plus 3-tier graduated lyrics dimming with accent+bold current line, primary near lines, and dim far lines**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-26T15:54:40Z
- **Completed:** 2026-03-26T15:57:44Z
- **Tasks:** 2 of 2 auto tasks (Task 3 is checkpoint:human-verify)
- **Files modified:** 1

## Accomplishments
- Two-column metadata layout uses Fill(1)+Fill(1) with inter-column spacing, eliminating the 1-pixel gap at odd terminal widths from Percentage(50) rounding
- Lyrics now have 3-tier graduated dimming: current line in accent+bold, +-2 near lines in primary, far lines in dim
- Safe distance calculation using isize cast + unsigned_abs avoids unsigned underflow when current index > iteration index

## Task Commits

Each task was committed atomically:

1. **Task 1: Align metadata and fix two-column rounding** - `cd8b82a` (feat)
2. **Task 2: Add 3-tier graduated lyrics dimming** - `8861ab1` (feat)

**Task 3:** checkpoint:human-verify (awaiting user visual verification)

**Plan metadata:** (pending final commit after checkpoint)

## Files Created/Modified
- `src/ui/mod.rs` - Replaced Percentage(50)+Percentage(50) with Fill(1)+Fill(1) in draw_metadata, replaced binary highlight with 3-tier graduated dimming in draw_lyrics

## Decisions Made
- Used Fill(1)+Fill(1) with spacing(SPACING_NORMAL) -- same visual intent as Percentage(50) but no rounding gap at odd widths
- Used theme.accent (not theme.primary) for current lyrics line -- accent is brighter/warmer than primary across all 6 themes per research findings
- Distance threshold of 2 for near lines -- provides visible gradient without too many tiers
- isize cast + unsigned_abs -- safe approach that avoids panic on unsigned subtraction underflow

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All automated tasks for Phase 3 complete (Plans 01 + 02)
- Awaiting human visual verification (Task 3 checkpoint) to confirm all four section changes look correct
- Full make verify passes: fmt clean, clippy clean, 11 tests pass, release build succeeds

## Self-Check: PENDING

Self-check will be finalized after checkpoint resolution.

---
*Phase: 03-section-level-polish*
*Completed: 2026-03-26 (pending checkpoint)*
