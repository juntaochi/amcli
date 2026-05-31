1. **Optimize formatting in `App::update`**
   - Inline the integer calculations and formatting from `format_duration` and `format_duration_seconds` directly into `cache.duration_str` and `cache.gauge_label`.
   - This eliminates 4 intermediate `String` allocations per tick.
   - Remove the now-unused `format_duration` and `format_duration_seconds` functions.
2. **Run lint and tests**
   - Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`.
3. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
4. **Submit PR**
   - Use branch `bolt-perf-duration-fmt` and create a PR detailing the optimization and its impact.
