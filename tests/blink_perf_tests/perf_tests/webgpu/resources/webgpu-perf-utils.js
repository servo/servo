
function skipTest(message) {
  PerfTestRunner.log(message);

  const skip = () => {
    if (window.testRunner) {
      testRunner.notifyDone();
    }
  }

  if (window.testRunner && window.testRunner.telemetryIsRunning) {
    testRunner.waitForTelemetry([], skip);
  } else {
    skip();
  }
}
