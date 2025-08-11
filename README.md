# youtube_wait

- If you want the cycle to be exactly 60s or 120s instead of 90s, change
  cycle_timer: Timer::from_seconds(90.0, ...) in src/main.rs.
  • files touched: src/main.rs
  • notable constants:
    • start_zoom: 3.0
    • min_zoom: 0.0006
    • cycle_timer: 90.0 seconds
    • target centers in targets array
  • where to tweak:
    • Increase visible depth: reduce min_zoom slightly and raise the max_iter cap.
    • Slow/fast zoom: adjust cycle_timer seconds.

