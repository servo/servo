function jank(ms) {
  let start = performance.now();
  while (performance.now() < start + ms);
}

function jankMany(ms, times) {
  for (let i = 0; i < times; i++) {
    jank(ms);
  }
}
