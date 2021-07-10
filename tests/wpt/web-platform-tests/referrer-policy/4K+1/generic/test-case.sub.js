function TestCase(scenarios, sanityChecker) {
  function runTest(scenario) {
    // This check is A NOOP in release.
    sanityChecker.checkScenario(scenario);

    runLengthTest(
        scenario,
        4096 + 1,
        "origin",
        scenario.test_description);
  }

  function runTests() {
    for (const scenario of scenarios) {
      runTest(scenario);
    }
  }

  return {start: runTests};
}
