# Phase 1: Draw Function Decomposition - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning
**Mode:** Auto-generated (infrastructure phase — discuss skipped)

<domain>
## Phase Boundary

Each UI section (artwork, metadata, lyrics, progress, controls, chassis) renders through its own isolated function, enabling safe per-section modifications in later phases. The monolithic 455-line draw() function in src/ui/mod.rs must be decomposed into an orchestrator that computes layout Rects and delegates to section-specific renderer functions.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion
All implementation choices are at Claude's discretion — pure infrastructure phase. Use ROADMAP phase goal, success criteria, and codebase conventions to guide decisions.

Key guidance from research:
- Use free-function section renderers (not custom Widget structs) — internal sections, not reusable
- Pass narrow data slices (Frame, Rect, theme, specific fields) — not &mut App
- Existing draw_lyrics() and settings.render() serve as the pattern to follow
- Extract leaf sections first (controls, progress, idle), then complex ones (artwork, metadata)
- Application must render identically to before — no visual changes

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `draw_lyrics()` — already extracted as a separate function, serves as the extraction pattern
- `settings.render()` — settings overlay already isolated in src/ui/settings.rs
- Theme constants (THEMES array, color definitions) — used across all sections

### Established Patterns
- Ratatui Layout::default().direction().constraints().split() for area computation
- Block::default().borders().border_style() for section framing
- Paragraph, Gauge, Line, Span for content rendering
- StatefulProtocol for artwork rendering (needs &mut — handle with care)

### Integration Points
- draw() is called from run_app() in main.rs via terminal.draw(|f| draw(f, &mut app))
- App struct provides all state; draw() takes &mut App for StatefulProtocol and ThrobberState
- Settings overlay renders on top of everything (z-order via render order)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — infrastructure phase. Refer to ROADMAP phase description and success criteria.

</specifics>

<deferred>
## Deferred Ideas

None — infrastructure phase.

</deferred>
