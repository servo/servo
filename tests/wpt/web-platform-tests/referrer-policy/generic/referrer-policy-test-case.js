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

// NOTE: This method only strips the fragment and is not in accordance to the
// recommended draft specification:
// https://w3c.github.io/webappsec/specs/referrer-policy/#null
// TODO(kristijanburnik): Implement this helper as defined by spec once added
// scenarios for URLs containing username/password/etc.
function stripUrlForUseAsReferrer(url) {
  return url.replace(/#.*$/, "");
}

function ReferrerPolicyTestCase(scenario, testDescription, sanityChecker) {
  // Pass and skip rest of the test if browser does not support fetch.
  if (scenario.subresource == "fetch-request" && !window.fetch) {
    // TODO(kristijanburnik): This should be refactored.
    return {
      start: function() {
        test(function() { assert_true(true); },
             "[ReferrerPolicyTestCase] Skipping test: Fetch is not supported.");
      }
    };
  }

  // This check is A NOOP in release.
  sanityChecker.checkScenario(scenario);

  const originTypeConversion = {
    "same-origin-http": "same-http",
    "same-origin-https": "same-https",
    "cross-origin-http": "cross-http",
    "cross-origin-https": "cross-https"
  };
  const urls = getRequestURLs(
      scenario.subresource,
      originTypeConversion[scenario.origin + '-' + scenario.target_protocol],
      scenario.redirection);
  const invoker =
      subresourceMap[scenario.subresource].invokerForReferrerPolicy ||
      subresourceMap[scenario.subresource].invoker;
  const checkResult = result => {
    const referrerUrlResolver = {
      "omitted": function() {
        return undefined;
      },
      "origin": function() {
        return self.origin + "/";
      },
      "stripped-referrer": function() {
        return stripUrlForUseAsReferrer(location.toString());
      }
    };
    const expectedReferrerUrl =
      referrerUrlResolver[scenario.referrer_url]();

    // Check if the result is in valid format. NOOP in release.
    sanityChecker.checkSubresourceResult(scenario, urls.testUrl, result);

    // Check the reported URL.
    assert_equals(result.referrer,
                  expectedReferrerUrl,
                  "Reported Referrer URL is '" +
                  scenario.referrer_url + "'.");
    assert_equals(result.headers.referer,
                  expectedReferrerUrl,
                  "Reported Referrer URL from HTTP header is '" +
                  expectedReferrerUrl + "'");
  };

  function runTest() {
    promise_test(_ => {
      // Depending on the delivery method, extend the subresource element with
      // these attributes.
      var elementAttributesForDeliveryMethod = {
        "attr-referrer":  {referrerPolicy: scenario.referrer_policy},
        "rel-noreferrer": {rel: "noreferrer"}
      };
      var deliveryMethod = scenario.delivery_method;
      let elementAttributes = {};
      if (deliveryMethod in elementAttributesForDeliveryMethod) {
        elementAttributes = elementAttributesForDeliveryMethod[deliveryMethod];
      }
      return invoker(urls.testUrl, elementAttributes, scenario.referrer_policy)
        .then(checkResult);
    }, testDescription);
  }

  return {start: runTest};
}
