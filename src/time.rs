use std::time::Duration;

#[cfg(target_arch="wasm32")] 
pub fn sleep(duration: Duration) {
  // just spin since i dont know how to sleep in wasm and digging through
  // emscripten's libc isn't getting me anywhere.
  // i have no sleep and i must wait
  use web_time::Instant;

  let loop_start = Instant::now();

  while loop_start.elapsed() < duration {}
}

#[cfg(not(target_arch="wasm32"))]
pub fn sleep(duration: Duration) {
  let start = std::time::Instant::now();

  // sleep for 95% (19/20) of the duration (this percentage is also used in raylib)
  std::thread::sleep(duration * 19 / 20);

  while start.elapsed() < duration {}
}