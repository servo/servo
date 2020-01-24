function TestCase(scenario, testDescription, sanityChecker) {
  // This check is A NOOP in release.
  sanityChecker.checkScenario(scenario);
  return {
    start: () => runLengthTest(
        scenario,
        4096 + 1,
        "origin",
        "`Referer` header with length > 4k is stripped to an origin.")
  };
}
