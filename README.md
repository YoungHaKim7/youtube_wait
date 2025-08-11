# youtube_wait

Done: removed Local::now and made time controllable

  • Replaced Local::now() usages with a new AppClock resource you can control.
  • Added systems:
    • setup_target_time and setup_clock (chained so clock sees target).
    • advance_clock updates AppClock.current each frame based on Time and
      AppClock.speed.
    • update_countdown_text now uses AppClock.current.
  • Confirmed clean build.

  How to control time:
  • Set START_TIME_STR in src/main.rs to a fixed start (e.g.,
    "2025-12-31T23:54:59") or leave None to start 5 minutes before target.
  • Adjust AppClock.speed (e.g., set to 0.0 to freeze, 2.0 for 2x).
  • You can also toggle AppClock.paused.

  Key files touched:
  • src/main.rs: added AppClock, replaced Local::now(), added advance_clock,
    reordered systems.
  • README.md: documented the controllable clock.

  If you want runtime controls (keyboard to pause/speed up), I can add an input
  system next.

  ~ src/main.rs (+58/-6)
  ~ README.md (+9/-3)

<hr />

• Replaced deprecated chrono parsing and fixed timezone math:
    • Switched from Local.datetime_from_str(...) to
      NaiveDateTime::parse_from_str(...).and_local_timezone(Local).single().
    • Introduced controllable clock; removed all direct Local::now() usage.
  • Updated deprecated Bevy query usage:
    • Replaced get_single_mut with single_mut() and handled the Result with if let 
      Ok(...).
  • Clean build confirmed.

  Key edits in src/main.rs:
  • Use statement now imports NaiveDateTime and Duration.
  • Added `AppClock` resource and `advance_clock` system.
  • Countdown computes against `AppClock.current` (injectable), not Local::now().
  • Window/sprite and text queries now use single_mut().

  You can run the app now; the warnings and the E0277 error are resolved.

  ~ src/main.rs (+8/-8)


- If you want the cycle to be exactly 60s or 120s instead of 90s, change
  cycle_timer: Timer::from_seconds(90.0, ...) in src/main.rs.
 
 Controlling time:
 - Set `START_TIME_STR` (e.g., "2025-12-31T23:54:59") to pick the app's initial local time.
 - Adjust `AppClock.speed` to run faster/slower than real time (1.0 = realtime).
 - Toggle `AppClock.paused` to freeze the countdown.
  • files touched: src/main.rs
  • notable constants:
    • start_zoom: 3.0
    • min_zoom: 0.0006
    • cycle_timer: 90.0 seconds
    • target centers in targets array
  • where to tweak:
    • Increase visible depth: reduce min_zoom slightly and raise the max_iter cap.
    • Slow/fast zoom: adjust cycle_timer seconds.

