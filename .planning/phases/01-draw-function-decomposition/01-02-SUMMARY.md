---
phase: 01-draw-function-decomposition
plan: 02
subsystem: ui
tags: [ratatui, refactor, draw-decomposition, borrow-checker, artwork-renderer, metadata-renderer]

# Dependency graph
requires:
  - phase: 01-01
    provides: Six leaf-level section renderer functions with narrow data-slice signatures
provides:
  - draw_artwork standalone function with narrow mutable slices (StatefulProtocol, ThrobberState)
  - draw_metadata standalone function receiving &Track (eliminating per-frame clone)
  - Fully decomposed draw() orchestrator (99 lines, 8 section renderer dispatches)
  - Phase 1 complete -- draw() is a pure layout-computation + dispatch function
affects: [02-layout-foundation, 03-per-section-polish]

# Tech tracking
tech-stack:
  added: []
  patterns: [mutable-borrow-scoping, field-destructuring-for-borrow-checker, clone-elimination-via-ref]

key-files:
  created: []
  modified:
    - src/ui/mod.rs

key-decisions:
  - "Artwork mutable borrows scoped before immutable borrows to satisfy borrow checker"
  - "Track clone eliminated: app.current_track.as_ref() instead of app.get_current_track().cloned()"
  - "Layout constraints extracted to shared slice variable to reduce draw() line count"

patterns-established:
  - "Mutable-borrow scoping: draw_artwork (mutates throbber_state, artwork_protocol) called before draw_metadata/draw_lyrics (immutable borrows)"
  - "Clone elimination: pass &Track references through function parameters instead of cloning owned values"

requirements-completed: [STRC-01, STRC-02]

# Metrics
duration: 10min
completed: 2026-03-26
---

# Phase 01 Plan 02: Artwork/Metadata Extraction and Orchestrator Cleanup Summary

**Extracted draw_artwork and draw_metadata with narrow mutable/immutable slices, reducing draw() from 454 lines to a 99-line pure orchestrator dispatching 8 section renderers**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-26T14:13:40Z
- **Completed:** 2026-03-26T14:23:59Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Extracted draw_artwork as standalone function with narrow mutable slices (Option<&mut StatefulProtocol>, &mut ThrobberState) instead of &mut App
- Extracted draw_metadata receiving &Track reference, eliminating per-frame Track clone from the hot render path
- Cleaned draw() into a 99-line pure orchestrator: layout computation + 8 function call dispatches, zero inline widget construction
- Borrow order enforced: mutable artwork borrows scoped before immutable metadata/lyrics borrows
- All 11 tests pass, zero clippy warnings, make verify passes

## Task Commits

Each task was committed atomically:

1. **Task 1: Extract draw_artwork and draw_metadata functions** - `4a54636` (refactor)
2. **Task 2: Clean up draw() orchestrator and run full verification** - `8b676b4` (refactor)

## Files Created/Modified
- `src/ui/mod.rs` - draw_artwork and draw_metadata extracted; draw() reduced to 99-line orchestrator with 8 section renderer dispatches

## Decisions Made
- Artwork mutable borrows (throbber_state, artwork_protocol) scoped before immutable borrows to satisfy Rust borrow checker without unsafe code
- Track clone eliminated by changing app.get_current_track().cloned() to app.current_track.as_ref(), passing &Track through function parameters
- Layout constraints extracted to a shared slice variable to compact the orchestrator below 100 lines
- Inline borrow comment removed in favor of code structure (mutable calls physically precede immutable calls)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Applied rustfmt formatting to draw_metadata**
- **Found during:** Task 2 verification (make verify)
- **Issue:** Paragraph::new(lines).block() call in draw_metadata didn't match rustfmt expectations
- **Fix:** Ran cargo fmt to auto-format the render_widget call
- **Files modified:** src/ui/mod.rs
- **Verification:** make verify passes (fmt check + clippy + test + build)
- **Committed in:** 8b676b4

---

**Total deviations:** 1 auto-fixed (1 formatting)
**Impact on plan:** Trivial formatting adjustment. No scope creep.

## Issues Encountered
- draw() line count was initially 118 lines (above the 100-line acceptance criterion). Compacted by: extracting shared constraint slice, tuple destructuring for chunks, removing non-essential blank lines. Final count: 99 lines.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 1 (draw-function-decomposition) is COMPLETE
- All 8 section renderers exist with narrow data-slice signatures
- draw() is a pure orchestrator: computes layout Rects, dispatches to section functions
- No function takes &App or &mut App (except draw() itself for field access)
- Phase 2 (layout-foundation) can now modify individual section renderers independently
- Phase 3 (per-section-polish) can tune each renderer without touching the orchestrator

## Self-Check: PASSED

- All files exist (src/ui/mod.rs, 01-02-SUMMARY.md)
- All commits verified (4a54636, 8b676b4)
- 8 section renderer functions confirmed (fn draw_ count = 8)
- draw() function body = 99 lines (under 100)

---
*Phase: 01-draw-function-decomposition*
*Completed: 2026-03-26*
