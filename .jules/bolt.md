## 2024-05-22 - TUI Rendering Optimization
**Learning:** In TUI applications (Ratatui), helper functions called in the render loop (like `scroll_text`) run frequently. Avoiding intermediate allocations (`format!`, `Vec<char>`) by using iterator adapters (`chain`, `cycle`) can yield significant speedups (e.g., 32%).
**Action:** Inspect `draw` functions for repetitive allocations and replace them with zero-allocation iterators where possible.

## 2025-05-19 - AppleScript Process Spawning Overhead
**Learning:** This application relies heavily on `osascript` to communicate with Apple Music. Each call spawns a new process, which is expensive. The `AppleMusicController` was making redundant calls (fetching track info inside `get_artwork_url` even though `App` just fetched it).
**Action:** Minimize `osascript` calls by batching data retrieval or passing cached data between components. Implemented caching for artwork URL lookups to avoid both the `osascript` call and the HTTP request.
