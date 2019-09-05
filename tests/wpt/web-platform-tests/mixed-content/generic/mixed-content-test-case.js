/**
 * @fileoverview Test case for mixed-content in Web Platform Tests.
 * @author burnik@google.com (Kristijan Burnik)
 */

// TODO: This function is currently placed and duplicated at:
// - mixed-content/generic/mixed-content-test-case.js
// - referrer-policy/generic/referrer-policy-test-case.js
// but should be moved to /common/security-features/resources/common.js.
function getSubresourceOrigin(originType) {
  const httpProtocol = "http";
  const httpsProtocol = "https";
  const wsProtocol = "ws";
  const wssProtocol = "wss";

  const sameOriginHost = "{{host}}";
  const crossOriginHost = "{{domains[www1]}}";

  // These values can evaluate to either empty strings or a ":port" string.
  const httpPort = getNormalizedPort(parseInt("{{ports[http][0]}}", 10));
  const httpsPort = getNormalizedPort(parseInt("{{ports[https][0]}}", 10));
  const wsPort = getNormalizedPort(parseInt("{{ports[ws][0]}}", 10));
  const wssPort = getNormalizedPort(parseInt("{{ports[wss][0]}}", 10));

  /**
    @typedef OriginType
    @type {string}

    Represents the origin of the subresource request URL.
    The keys of `originMap` below are the valid values.

    Note that there can be redirects from the specified origin
    (see RedirectionType), and thus the origin of the subresource
    response URL might be different from what is specified by OriginType.
  */
  const originMap = {
    "same-https": httpsProtocol + "://" + sameOriginHost + httpsPort,
    "same-http": httpProtocol + "://" + sameOriginHost + httpPort,
    "cross-https": httpsProtocol + "://" + crossOriginHost + httpsPort,
    "cross-http": httpProtocol + "://" + crossOriginHost + httpPort,
    "same-wss": wssProtocol + "://" + sameOriginHost + wssPort,
    "same-ws": wsProtocol + "://" + sameOriginHost + wsPort,
    "cross-wss": wssProtocol + "://" + crossOriginHost + wssPort,
    "cross-ws": wsProtocol + "://" + crossOriginHost + wsPort,
  };

  return originMap[originType];
}

/**
 * MixedContentTestCase exercises all the tests for checking browser behavior
 * when resources regarded as mixed-content are requested. A single run covers
 * only a single scenario.
 * @param {object} scenario A JSON describing the test arrangement and
 *     expectation(s). Refer to /mixed-content/spec.src.json for details.
 * @param {string} description The test scenario verbose description.
 * @param {SanityChecker} sanityChecker Instance of an object used to check the
 *     running scenario. Useful in debug mode. See ./sanity-checker.js.
 *     Run {@code ./tools/generate.py -h} for info on test generating modes.
 * @return {object} Object wrapping the start method used to run the test.
 */
function MixedContentTestCase(scenario, description, sanityChecker) {
  sanityChecker.checkScenario(scenario, subresourceMap);

  let sourceContextList = [];
  let subresourceType = scenario.subresource;
  if (subresourceType === 'classic-data-worker-fetch') {
    // Currently 'classic-data-worker-fetch' (fetch API from inside classic
    // data: worker) is handled as a kind of subresource request
    // on the genarator side, but should be processed using the combination of
    // SourceContext list (classic data: worker) + Subresource (fetch API)
    // on the JavaScript side.
    // We bridge this inconsistency here, and will later pass these information
    // directly from the generated tests and remove this conversion here.
    subresourceType = 'fetch-request';
    sourceContextList = [{sourceContextType: 'worker-classic-data'}];
  }

  const originTypeConversion = {
    "same-host-https": "same-https",
    "same-host-http": "same-http",
    "cross-origin-https": "cross-https",
    "cross-origin-http": "cross-http",
    "same-host-wss": "same-wss",
    "same-host-ws": "same-ws",
    "cross-origin-wss": "cross-wss",
    "cross-origin-ws": "cross-ws",
  };

  const urls = getRequestURLs(subresourceType,
                              originTypeConversion[scenario.origin],
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
      subresourceType: subresourceType,
      url: urls.testUrl,
      policyDeliveries: []
    };

    promise_test(() => {
      return xhrRequest(urls.announceUrl)
        // Send out the real resource request.
        // This should tear down the key if it's not blocked.
        .then(_ => invokeRequest(subresource, sourceContextList))
        // We check the key state, regardless of whether the main request
        // succeeded or failed.
        .then(checkResult, checkResult);
      }, description);
  }  // runTest

  return {start: runTest};
}  // MixedContentTestCase
