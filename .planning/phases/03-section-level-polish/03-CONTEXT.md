# Phase 3: Section-Level Polish - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Polish each individual section to its final visual quality: center artwork in its area, distribute control buttons evenly, align metadata labels/values, and add graduated lyrics highlighting with dimming.

</domain>

<decisions>
## Implementation Decisions

### Artwork Centering
- Center artwork both vertically and horizontally in its area
- Use Ratatui Rect::centered() or manual padding split — whatever works with StatefulProtocol's &mut requirements

### Button Distribution
- Use Flex::SpaceBetween for edge-to-edge button distribution
- Use Constraint::Fill(1) per button for equal width — fills the line completely
- Eliminates the remainder gap visible in the current manual width/count division

### Metadata Alignment
- Song info labels (曲名/アーティスト/アルバム) and values cleanly aligned in consistent column layout
- Claude's discretion on exact alignment approach (fixed-width labels, table layout, etc.)

### Lyrics Visual Treatment
- Current line: Theme accent color (fg) + Bold modifier
- Near lines (±1-2 from current): Normal fg color
- Far lines (beyond ±2): Dimmed fg color (Dark variant or reduced intensity)
- 3-level graduated dimming system

### Claude's Discretion
- Exact centering implementation for artwork (Rect::centered vs manual padding vs Flex::Center)
- Metadata column alignment approach
- Specific dimming color values per theme
- Whether to modify draw_lyrics signature or add dimming inside existing function

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- draw_artwork() renderer — already isolated, receives area Rect
- draw_controls() renderer — already isolated, receives area Rect
- draw_metadata() renderer — already isolated
- draw_lyrics() renderer — already isolated
- Theme accent color available as theme.accent
- SPACING_NORMAL/SPACING_SECTION constants available

### Established Patterns
- Section renderers take (Frame, Rect, specific_data) — modify internal layout only
- Ratatui 0.30: Rect::centered(), Layout::horizontal().flex(), Constraint::Fill()
- Flex::SpaceBetween distributes constraints edge-to-edge

### Integration Points
- draw_artwork() in src/ui/mod.rs — centering changes
- draw_controls() in src/ui/mod.rs — button distribution changes
- draw_metadata() in src/ui/mod.rs — alignment changes
- draw_lyrics() in src/ui/mod.rs — highlighting/dimming changes
- All changes are within isolated section renderers — orchestrator unchanged

</code_context>

<specifics>
## Specific Ideas

- The screenshot showed artwork pinned to top-left of its area — centering should make it float in the middle
- Button remainder gap is visible at certain widths — Fill(1) + SpaceBetween eliminates it completely
- Current lyrics have a highlight but it lacks visual weight — accent color + Bold gives more contrast

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>
