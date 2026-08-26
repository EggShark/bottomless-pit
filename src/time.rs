use std::time::Duration;

pub fn sleep(duration: Duration) {
  // just spin since i dont know how to sleep in wasm and digging through
  // emscripten's libc isn't getting me anywhere.
  // i have no sleep and i must wait
  #[cfg(target_arch="wasm32")]
  {
    use web_time::Instant;

    let loop_start = Instant::now();

    while loop_start.elapsed() < duration {}

    return;
  }

  let start = std::time::Instant::now();
  
  // sleep for 95% of the duration (this percentage is also used in raylib)
  std::thread::sleep(duration / 20 * 19);

  while start.elapsed() < duration {}
}