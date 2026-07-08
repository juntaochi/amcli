# Roadmap: AMCLI UI Responsiveness & Polish

## Overview

Transform AMCLI's working-but-rough TUI into a polished layout that fills and centers content gracefully at any terminal size. The monolithic draw function must be decomposed first (safe modification), then a spacing system and proportional layout applied (foundation), then individual sections polished (artwork centering, button distribution, lyrics highlight). Three phases, each delivering a verifiable capability that enables the next.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Draw Function Decomposition** - Break monolithic draw() into orchestrator + isolated section renderers
- [x] **Phase 2: Proportional Layout & Spacing System** - Replace legacy constraints with Fill-based proportional layout and unified spacing constants
- [x] **Phase 3: Section-Level Polish** - Center artwork, distribute buttons, align metadata, highlight lyrics

## Phase Details

### Phase 1: Draw Function Decomposition
**Goal**: Each UI section (artwork, metadata, lyrics, progress, controls, chassis) renders through its own isolated function, enabling safe per-section modifications in later phases
**Depends on**: Nothing (first phase)
**Requirements**: STRC-01, STRC-02
**Success Criteria** (what must be TRUE):
  1. The draw() function delegates to section-specific renderer functions rather than containing all rendering inline
  2. Each section renderer receives a Rect and narrow data slices (not the entire App struct)
  3. The application renders identically to before decomposition -- no visual changes
  4. Settings overlay renders correctly on top of all sections (z-order preserved)
**Plans:** 2 plans

Plans:
- [x] 01-01-PLAN.md -- Extract leaf section renderers (controls, progress, idle, chassis, screen_border, lyrics narrowing)
- [x] 01-02-PLAN.md -- Extract artwork + metadata renderers, finalize draw() orchestrator

### Phase 2: Proportional Layout & Spacing System
**Goal**: Layout fills available terminal space proportionally with consistent spacing, eliminating dead zones and ad-hoc padding
**Depends on**: Phase 1
**Requirements**: LAYT-03, LAYT-04, VISL-01
**Success Criteria** (what must be TRUE):
  1. Resizing the terminal shows content areas growing and shrinking proportionally -- no fixed-size gaps or dead zones appear
  2. The artwork/info column split adapts to terminal width (artwork gets more space in wide terminals, info gets protected minimum in narrow ones)
  3. Spacing between all sections (artwork, info, lyrics, progress, controls) is visually consistent -- no section has noticeably tighter or looser margins than others
**Plans:** 1 plan

Plans:
- [x] 02-01-PLAN.md -- Spacing constants, Fill-based proportional layout, and Layout::spacing() integration

### Phase 3: Section-Level Polish
**Goal**: Each individual section reaches visual polish -- artwork centered, buttons evenly distributed, metadata aligned, lyrics highlighted with graduated dimming
**Depends on**: Phase 2
**Requirements**: LAYT-01, LAYT-02, VISL-02, VISL-03, VISL-04
**Success Criteria** (what must be TRUE):
  1. Album artwork appears vertically centered in its area (not pinned to the top-left corner)
  2. Control buttons are evenly distributed across the full terminal width at any size -- no remainder gap on the right
  3. Song info labels and values (title, artist, album) are cleanly aligned in a consistent column layout
  4. The current lyrics line is visually distinct (highlighted with accent color), and surrounding lines dim gradually with distance from the current line
**Plans:** 2 plans
**UI hint**: yes

Plans:
- [x] 03-01-PLAN.md -- Center artwork with Rect::centered(), distribute buttons with Fill(1)
- [x] 03-02-PLAN.md -- Align metadata columns, add 3-tier graduated lyrics dimming

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3

**Status note (2026-07-08):** Phase checkboxes were reconciled against phase summaries and `src/ui/mod.rs`. Automated implementation and `make verify` evidence exist for all three phases; the remaining checkpoint is human visual verification of Phase 3 polish.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Draw Function Decomposition | 2/2 | Complete | 2026-03-26 |
| 2. Proportional Layout & Spacing System | 1/1 | Complete | 2026-03-26 |
| 3. Section-Level Polish | 2/2 | Complete (automated); human visual verification pending | 2026-03-26 |
