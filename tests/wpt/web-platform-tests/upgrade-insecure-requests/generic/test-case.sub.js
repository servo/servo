// TODO(hiroshige): Document the type of `scenario`.
function TestCase(scenario, description) {
  const urls = getRequestURLs(scenario.subresource,
                              scenario.origin,
                              scenario.redirection);
  const checkResult = _ => {
    // Send request to check if the key has been torn down.
    return xhrRequest(urls.assertUrl)
      .then(assertResult => {
          // Now check if the value has been torn down. If it's still there,
          // we have blocked the request to mixed-content.
          assert_equals(assertResult.status, scenario.expectation,
            "The resource request should be '" + scenario.expectation + "'.");
        });
  };

  function runTest() {
    /** @type {Subresource} */
    const subresource = {
      subresourceType: scenario.subresource,
      url: urls.testUrl,
      policyDeliveries: scenario.subresource_policy_deliveries,
    };

    promise_test(() => {
      return xhrRequest(urls.announceUrl)
        // Send out the real resource request.
        // This should tear down the key if it's not blocked.
        .then(_ => invokeRequest(subresource, scenario.source_context_list))
        // We check the key state, regardless of whether the main request
        // succeeded or failed.
        .then(checkResult, checkResult);
      }, description);
  }  // runTest

  return {start: runTest};
}
