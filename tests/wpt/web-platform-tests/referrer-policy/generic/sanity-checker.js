// The SanityChecker is used in debug mode to identify problems with the
// structure of the testsuite. In release mode it is mocked out to do nothing.

function SanityChecker()  {}

SanityChecker.prototype.checkScenario = function(scenario) {
  // Check if scenario is valid.
  // TODO(kristijanburnik): Move to a sanity-checks.js for debug mode only.
  test(function() {

    // We extend the exsiting test_expansion_schema not to kill performance by
    // copying.
    var expectedFields = SPEC_JSON["test_expansion_schema"];
    expectedFields["referrer_policy"] = SPEC_JSON["referrer_policy_schema"];

    for (var field in expectedFields) {
      assert_own_property(scenario, field,
                          "The scenario contains field " + field)
      assert_in_array(scenario[field], expectedFields[field],
                      "Scenario's " + field + " is one of: " +
                      expectedFields[field].join(", ")) + "."
    }

    // Check if the protocol is matched.
    assert_equals(scenario["source_protocol"] + ":", location.protocol,
                  "Protocol of the test page should match the scenario.")

  }, "[ReferrerPolicyTestCase] The test scenario is valid.");
}

SanityChecker.prototype.checkSubresourceResult = function(test,
                                                          scenario,
                                                          subresourceUrl,
                                                          result) {
  test.step(function() {
    assert_equals(Object.keys(result).length, 3);
    assert_own_property(result, "location");
    assert_own_property(result, "referrer");
    assert_own_property(result, "headers");

    // Skip location check for scripts.
    if (scenario.subresource == "script-tag")
      return;

    // Sanity check: location of sub-resource matches reported location.
    assert_equals(result.location, subresourceUrl,
                  "Subresource reported location.");
  }, "Running a valid test scenario.");
};
