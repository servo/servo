export function pm(x) {
  const message = {value: x, checks: pm.checks};
  postMessage(message);
}
