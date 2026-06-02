/* Generate a slow task. */
const begin = window.performance.now();
while (window.performance.now() < begin + 60);
