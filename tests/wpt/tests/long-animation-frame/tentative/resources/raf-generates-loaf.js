requestAnimationFrame(() => {
  const deadline = performance.now() + 360;
  while (performance.now() < deadline) {
  }
});
