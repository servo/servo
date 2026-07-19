const very_long_frame_duration = 360;

function busy_wait(ms_delay = very_long_frame_duration) {
  const deadline = performance.now() + ms_delay;
  while (performance.now() < deadline) {
  }
}

function generate_long_animation_frame(duration = very_long_frame_duration) {
  busy_wait(duration / 2);
  const reference_time = performance.now();
  busy_wait(duration / 2);
  return new Promise(
      resolve => new PerformanceObserver((entries, observer) => {
                   const entry = entries.getEntries().find(
                       e => ((e.startTime < reference_time) &&
                             (reference_time < (e.startTime + e.duration))));
                   if (entry) {
                     observer.disconnect();
                     resolve(entry);
                   }
                 }).observe({type: 'long-animation-frame'}));
}
