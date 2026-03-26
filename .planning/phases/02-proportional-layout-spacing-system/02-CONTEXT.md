# Phase 2: Proportional Layout & Spacing System - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace legacy percentage-based constraints with Fill-based proportional layout and establish a unified spacing system. The orchestrator in draw() uses Constraint::Percentage(42)/Percentage(57) which sums to 99% leaving a 1-cell gap, ad-hoc spacing values throughout, and no adaptive min/max protection for artwork vs info columns.

</domain>

<decisions>
## Implementation Decisions

### Spacing System
- 3 spacing levels as constants at top of src/ui/mod.rs: TIGHT(0), NORMAL(1), SECTION(2)
- Apply spacing between artwork/info columns, between metadata/lyrics vertically, around progress bar and controls
- Constants defined as `const` block co-located with rendering code

### Proportional Layout
- Replace Constraint::Percentage(42)/Percentage(57) with Constraint::Fill(3)/Constraint::Fill(4) for artwork/info split — proportional, eliminates 1% gap
- Artwork column: Min(20) constraint to protect at narrow widths
- Info column: Min(30) constraint to protect at narrow widths
- Progress bar: Keep Length(3) — fixed content height
- Controls area: Keep Length(3) — fixed button row height

### Adaptive Split Logic
- Artwork visibility threshold: Keep current `width > 50` — reasonable cutoff
- Artwork/info ratio at wide terminals: 40/60 split (slightly favor info for lyrics readability)
- Two-column metadata threshold: Keep current logic (`metadata_width > 80 || (has_lyrics && info_height <= 14) && metadata_width >= 40`)

### Claude's Discretion
- Exact implementation of Layout::spacing() integration
- Whether to use Flex variants for any sub-layouts
- How to handle the metadata/lyrics vertical split proportionally

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- draw() orchestrator at lines 1001-1100 of src/ui/mod.rs (now 99 lines after Phase 1)
- 8 section renderers already extracted — can modify orchestrator layout without touching renderers
- Theme constants already defined at top of file

### Established Patterns
- Layout::default().direction().constraints().split() for area computation
- Constraint::Percentage, Constraint::Length, Constraint::Min used throughout
- Ratatui 0.30 has Constraint::Fill(), Rect::centered(), Layout::spacing() available but unused

### Integration Points
- Orchestrator draw() function (lines 1001-1100) — primary modification target
- Spacing constants will be referenced by section renderers if they have internal padding
- No new files needed — all changes in src/ui/mod.rs

</code_context>

<specifics>
## Specific Ideas

- The Percentage(42)/Percentage(57) sum-to-99% gap is visible as a 1-cell line at certain terminal widths
- Constraint::Fill(3)/Fill(4) gives roughly 43%/57% without any gap
- Layout::spacing(1) can add consistent inter-section gaps managed by the layout engine

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>
