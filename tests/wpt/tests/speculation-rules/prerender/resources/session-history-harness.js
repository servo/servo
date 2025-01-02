// We don't have the test harness in this context, so we roll our own
// which communicates with our initiator which is actually running the tests.

function assert(condition, message) {
  if (!condition) {
    throw new Error("Assertion failed: " + message);
  }
}

// Run a test after activation.
document.addEventListener("prerenderingchange", async (_) => {
  // history.length is racy on activation. Wait *100ms* as a workaround.
  // See crbug.com/1222893.
  await new Promise((resolve) => {
    window.setTimeout(resolve, 100);
  });

  const urlParams = new URLSearchParams(window.location.search);
  const testName = urlParams.get("testName");
  const uid = urlParams.get("uid");
  const testChannel = new PrerenderChannel(
    `test-channel-${testName}`, uid
  );

  try {
    const activationTestFn = testName + "Activation";
    const testFn = window[activationTestFn];
    if (!testFn) {
      testChannel.postMessage("Missing test: " + testName);
      return;
    }
    testFn();
    testChannel.postMessage("Passed");
  } catch (e) {
    testChannel.postMessage(
      "Failed: " + e.name + ": " + e.message,
    );
  } finally {
    testChannel.close();
  }
})

if (document.prerendering) {
  window.onload = async () => {
    const urlParams = new URLSearchParams(window.location.search);
    const testName = urlParams.get("testName");
    const uid = urlParams.get("uid");
    const prerenderChannel = new PrerenderChannel(
      `prerender-channel-${testName}`, uid
    );

    // The document load event is not finished at this point, so navigations
    // would be done with replacement. This interferes with our tests. We wait
    // for the next task before navigating to avoid this.
    await new Promise((resolve) => {
      window.setTimeout(resolve);
    });

    try {
      let testFn = window[testName];
      if (!testFn) {
        prerenderChannel.postMessage("Missing test: " + testName);
        return;
      }
      await testFn();
      prerenderChannel.postMessage("Passed");
    } catch (e) {
      prerenderChannel.postMessage(
        "Failed: " + e.name + ": " + e.message,
      );
    } finally {
      prerenderChannel.close();
    }
  };
}
