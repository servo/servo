window.requestAnimationFrame(function() {
  /* Generate a slow task. */
  var begin = window.performance.now();
  while (window.performance.now() < begin + 51);
});
