// We don't have the test harness in this context, so we roll our own
// which communicates with our host which is actually running the tests.

window.onload = async () => {
  let urlParams = new URLSearchParams(window.location.search);
  let testName = urlParams.get('testName');
  let testFn = window[testName];
  if (!testFn) {
    window.portalHost.postMessage('Missing test: ' + testName);
    return;
  }

  // The document load event is not finished at this point, so navigations
  // would be done with replacement. This interferes with our tests. We wait
  // for the next task before navigating to avoid this.
  await new Promise((resolve) => { window.setTimeout(resolve); });

  try {
    await testFn();
    window.portalHost.postMessage('Passed');
  } catch (e) {
    window.portalHost.postMessage(
        'Failed: ' + e.name + ': ' + e.message);
  }
};

function assert(condition, message) {
  if (!condition)
    throw new Error('Assertion failed: ' + message);
}
