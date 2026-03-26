---
phase: 02-proportional-layout-spacing-system
plan: 01
subsystem: ui
tags: [ratatui, layout, constraint-fill, spacing, proportional]

# Dependency graph
requires:
  - phase: 01-draw-function-decomposition
    provides: "Decomposed draw() orchestrator with 8 section renderers"
provides:
  - "SPACING_TIGHT/NORMAL/SECTION constants for unified layout gaps"
  - "Fill-based proportional layout replacing all Percentage splits in orchestrator"
  - "Layout::spacing() integration for consistent inter-section gaps"
  - "Min(20) narrow-terminal guard for artwork column"
  - "Modernized Layout::vertical/horizontal + .areas() pattern throughout orchestrator"
affects: [03-per-section-polish]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Layout::vertical/horizontal + .areas() destructuring instead of .default().direction().split() + indexing"
    - "Constraint::Fill(weight) for proportional space distribution instead of Percentage"
    - "Layout::spacing(SPACING_NORMAL) for inter-section gaps instead of manual Length(1) separators"
    - "SPACING_TIGHT/NORMAL/SECTION constant system for all spacing values"

key-files:
  created: []
  modified:
    - src/ui/mod.rs

key-decisions:
  - "Fill(3)/Fill(4) ratio for artwork/info split (~43%/57%) replacing Percentage(42)/Percentage(57)"
  - "Fill(2)/Fill(3) ratio for metadata/lyrics horizontal split (~40%/60%)"
  - "Conditional Min(20) guard at narrow widths instead of compound constraints"
  - "SPACING_TIGHT annotated #[allow(dead_code)] since it completes the spacing system but has no current use"

patterns-established:
  - "Spacing constants: SPACING_TIGHT(0), SPACING_NORMAL(1), SPACING_SECTION(2) defined with theme constants"
  - "Layout pattern: Layout::vertical/horizontal([constraints]).spacing(X).areas(rect) for all orchestrator layouts"
  - "Proportional pattern: Fill(weight) pairs for flexible splits, Length(n) for fixed-size areas"
  - "Narrow guard pattern: width threshold check selecting Fill or Min constraints conditionally"

requirements-completed: [LAYT-03, LAYT-04, VISL-01]

# Metrics
duration: 5min
completed: 2026-03-26
---

# Phase 2 Plan 1: Proportional Layout & Spacing System Summary

**Fill-based proportional layout with spacing constants replacing all Percentage splits and ad-hoc padding in the draw() orchestrator**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-26T14:50:01Z
- **Completed:** 2026-03-26T14:54:53Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Defined SPACING_TIGHT/NORMAL/SECTION constant system co-located with theme constants
- Replaced all Percentage-based splits with Fill-weighted proportional constraints (Fill(3)/Fill(4) artwork/info, Fill(2)/Fill(3) metadata/lyrics)
- Modernized all Layout calls in draw() to use Layout::vertical/horizontal + .areas() destructuring
- Applied Layout::spacing(SPACING_NORMAL) to artwork/info split, metadata/lyrics horizontal split, and metadata/lyrics vertical split
- Added Min(20) narrow-terminal guard for artwork column protection
- Replaced metadata renderer magic-number padding with SPACING_NORMAL and SPACING_SECTION constants
- Eliminated all legacy patterns: Percentage(42), Percentage(57), Percentage(45), Percentage(55), Percentage(100), Min(0), Min(10)
- draw() orchestrator reduced from 99 to 89 lines

## Task Commits

Each task was committed atomically:

1. **Task 1: Add spacing constants and modernize orchestrator layout** - `40630d0` (feat)
2. **Task 2: Modernize metadata/lyrics split and apply spacing constants to renderer padding** - `193dcb0` (feat)

## Files Created/Modified

- `src/ui/mod.rs` - Spacing constants, modernized Layout calls in draw() orchestrator, spacing-constant-based padding in draw_metadata

## Decisions Made

- Used Fill(3)/Fill(4) for artwork/info giving ~43%/57% ratio (matches prior Percentage(42)/Percentage(57) visually but eliminates the 1% gap)
- Used Fill(2)/Fill(3) for metadata/lyrics horizontal split giving ~40%/60% ratio (close to prior 45/55 but proportional)
- Added #[allow(dead_code)] to SPACING_TIGHT since it defines the system boundary but has no current consumer -- future phases can use it
- Applied conditional Min(20)/Fill(1) fallback at narrow widths instead of trying to combine Min+Fill on single constraints
- Did not add Layout::spacing to main vertical layout (display/tuner/controls) since bordered blocks already provide visual separation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added #[allow(dead_code)] for clippy compliance**
- **Found during:** Task 1
- **Issue:** SPACING_TIGHT and SPACING_SECTION constants trigger clippy dead_code warning before Task 2 adds usage of SPACING_SECTION
- **Fix:** Added #[allow(dead_code)] on SPACING_TIGHT (permanent -- system completeness, no current consumer) and temporarily on SPACING_SECTION (removed in Task 2 when it gained usage)
- **Files modified:** src/ui/mod.rs
- **Verification:** make verify passes with zero clippy warnings
- **Committed in:** 40630d0 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minor annotation to satisfy clippy dead_code warnings. No scope creep.

## Issues Encountered

- cargo fmt reformatted the Padding::new() calls with spacing constants into multi-line format due to longer constant names. No functional impact.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Spacing constants and proportional layout foundation complete
- Phase 3 (per-section polish) can now:
  - Use SPACING_TIGHT/NORMAL/SECTION constants for consistent spacing in section renderers
  - Apply Layout::horizontal/vertical + .areas() pattern to controls and artwork renderers
  - Artwork centering (LAYT-01) and button distribution (LAYT-02) ready to implement using the established patterns

## Self-Check: PASSED

- src/ui/mod.rs: FOUND
- 02-01-SUMMARY.md: FOUND
- Commit 40630d0: FOUND
- Commit 193dcb0: FOUND

---
*Phase: 02-proportional-layout-spacing-system*
*Completed: 2026-03-26*
