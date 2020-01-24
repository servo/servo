function TestCase(scenario, testDescription, sanityChecker) {
  // This check is A NOOP in release.
  sanityChecker.checkScenario(scenario);
  return {
    start: () => runLengthTest(
        scenario,
        4096 - 1,
        scenario.expectation,
        "`Referer` header with length < 4k is not stripped to an origin.")
  };
}
