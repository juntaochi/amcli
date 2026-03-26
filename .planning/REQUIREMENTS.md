# Requirements: AMCLI UI Responsiveness & Polish

**Defined:** 2026-03-26
**Core Value:** The TUI looks polished and adapts gracefully to any terminal size

## v1 Requirements

### Code Structure

- [x] **STRC-01**: Draw function decomposed into orchestrator + section renderers with narrow data slices
- [x] **STRC-02**: Each section renderer is a standalone function receiving Frame, Rect, and relevant state

### Layout

- [ ] **LAYT-01**: Artwork vertically centered in its available area using Ratatui centering APIs
- [ ] **LAYT-02**: Control buttons evenly distributed across terminal width at any size using Flex layout
- [x] **LAYT-03**: Layout uses proportional Fill constraints instead of percentage-based splits that leave gaps
- [x] **LAYT-04**: Artwork/info split ratio adapts to terminal width using Min/Max constraints

### Visual Polish

- [x] **VISL-01**: Consistent spacing system with unified constants replacing ad-hoc padding values
- [ ] **VISL-02**: Song info area (title, artist, album) labels and values cleanly aligned
- [ ] **VISL-03**: Current lyrics line highlighted with stronger visual contrast
- [ ] **VISL-04**: Lyrics lines dim gradually with distance from current line (graduated dimming)

## v2 Requirements

### Adaptive Behaviors

- **ADPT-01**: Button labels truncate gracefully at very narrow terminal widths
- **ADPT-02**: Separator lines between major layout sections

## Out of Scope

| Feature | Reason |
|---------|--------|
| Graceful collapse/hide at tiny sizes | User wants fill & center at all sizes, not element hiding |
| Performance optimization | Not a current concern |
| New features (playlist, search, queue) | This milestone is layout/polish only |
| Responsive breakpoints | Proportional layout handles sizing continuously, not via breakpoints |
| Theme changes | Existing 6 themes unchanged; only spacing/layout affected |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| STRC-01 | Phase 1 | Complete |
| STRC-02 | Phase 1 | Complete |
| LAYT-01 | Phase 3 | Pending |
| LAYT-02 | Phase 3 | Pending |
| LAYT-03 | Phase 2 | Complete |
| LAYT-04 | Phase 2 | Complete |
| VISL-01 | Phase 2 | Complete |
| VISL-02 | Phase 3 | Pending |
| VISL-03 | Phase 3 | Pending |
| VISL-04 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
*Requirements defined: 2026-03-26*
*Last updated: 2026-03-26 after roadmap creation*
