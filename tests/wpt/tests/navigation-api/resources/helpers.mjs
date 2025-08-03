export async function ensureWindowLoadEventFired(t) {
  return new Promise(resolve => {
    const callback = () => t.step_timeout(resolve, 0);
    if (document.readyState === 'complete') {
      callback();
    } else {
      window.onload = callback;
    }
  });
}
