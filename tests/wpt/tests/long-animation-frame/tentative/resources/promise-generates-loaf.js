fetch("/common/dummy.xml").then(() => {
  const deadline = performance.now() + 360;
  while (performance.now() < deadline) {}
});
