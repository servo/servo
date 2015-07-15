// The SanityChecker is used in debug mode to identify problems with the
// structure of the testsuite. In release mode it is mocked out to do nothing.

function SanityChecker()  {}

SanityChecker.prototype.checkScenario = function(scenario, resourceInvoker) {
  // Check if scenario is valid.
  test(function() {
    var expectedFields = SPEC_JSON["test_expansion_schema"];

    for (var field in expectedFields) {
      if (field == "expansion")
        continue

      assert_own_property(scenario, field,
                          "The scenario should contain field '" + field + "'.")

      var expectedFieldList = expectedFields[field];
      if (!expectedFieldList.hasOwnProperty('length')) {
        var expectedFieldList = [];
        for (var key in expectedFields[field]) {
          expectedFieldList = expectedFieldList.concat(expectedFields[field][key])
        }
      }
      assert_in_array(scenario[field], expectedFieldList,
                      "Scenario's " + field + " is one of: " +
                      expectedFieldList.join(", ")) + "."
    }

    // Check if the protocol is matched.
    assert_equals(scenario["source_scheme"] + ":", location.protocol,
                  "Protocol of the test page should match the scenario.")

    assert_own_property(resourceInvoker, scenario.subresource,
                        "Subresource should be supported");

  }, "[MixedContentTestCase] The test scenario should be valid.");
}
