// Returns a promise that resolves with a Security Policy Violation (spv)
    // even when it is received.
function promise_spv() {
  return new Promise((resolve, reject) => {
    window.addEventListener("securitypolicyviolation", e => {
      resolve(e);
    }, { once: true });
  });
}
