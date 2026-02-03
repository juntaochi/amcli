## 2024-05-22 - TUI Rendering Optimization
**Learning:** In TUI applications (Ratatui), helper functions called in the render loop (like `scroll_text`) run frequently. Avoiding intermediate allocations (`format!`, `Vec<char>`) by using iterator adapters (`chain`, `cycle`) can yield significant speedups (e.g., 32%).
**Action:** Inspect `draw` functions for repetitive allocations and replace them with zero-allocation iterators where possible.
