// TODO: This function is currently placed and duplicated at:
// - mixed-content/generic/mixed-content-test-case.js
// - referrer-policy/generic/referrer-policy-test-case.sub.js
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

// NOTE: This method only strips the fragment and is not in accordance to the
// recommended draft specification:
// https://w3c.github.io/webappsec/specs/referrer-policy/#null
// TODO(kristijanburnik): Implement this helper as defined by spec once added
// scenarios for URLs containing username/password/etc.
function stripUrlForUseAsReferrer(url) {
  return url.replace(/#.*$/, "");
}

function invokeScenario(scenario, sourceContextList) {
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

  const deliveryTypeConversion = {
    "attr-referrer": "attr",
    "rel-noreferrer": "rel-noref",
    // Other delivery methods such as "http-rp" are ignored here because
    // they are already applied to the main document by generator.py.
  };

  /** @type {PolicyDelivery} */
  const delivery = {
      deliveryType: deliveryTypeConversion[scenario.delivery_method],
      key: "referrerPolicy",
      value: scenario.referrer_policy};

  /** @type {Subresource} */
  const subresource = {
    subresourceType: scenario.subresource,
    url: urls.testUrl,
    policyDeliveries: [delivery]
  };

  return invokeRequest(subresource, sourceContextList || []);
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

  const referrerUrlResolver = {
    "omitted": function(sourceUrl) {
      return undefined;
    },
    "origin": function(sourceUrl) {
      return new URL(sourceUrl).origin + "/";
    },
    "stripped-referrer": function(sourceUrl) {
      return stripUrlForUseAsReferrer(sourceUrl);
    }
  };

  const checkResult = (expectation, result) => {
    let currentURL = location.toString();
    const expectedReferrerUrl =
      referrerUrlResolver[expectation](currentURL);

    // Check the reported URL.
    assert_equals(result.referrer,
                  expectedReferrerUrl,
                  "Reported Referrer URL is '" +
                  expectation + "'.");
    assert_equals(result.headers.referer,
                  expectedReferrerUrl,
                  "Reported Referrer URL from HTTP header is '" +
                  expectedReferrerUrl + "'");
  };

  function runTest() {
    function historyBackPromise(t, scenario) {
      history.back();
      return new Promise(resolve => {
          // Wait for completion of `history.back()` by listening the
          // popstate events that are fired near the end of
          // `history.back()` processing.
          window.addEventListener('popstate', resolve, {once: true});

          // Workaround for Safari: Waiting for popstate events causes
          // timeout in a-tag tests. To avoid timeout, we anyway resolve
          // the promise.
          if (scenario.subresource === 'a-tag') {
            t.step_timeout(resolve, 1000);
          }
        });
    }

    // Request in the top-level document.
    promise_test(_ => {
      return invokeScenario(scenario)
        .then(result => checkResult(scenario.referrer_url, result));
    }, testDescription);

    // `Referer` headers with length over 4k are culled down to an origin, so,
    // let's test around that boundary for tests that would otherwise return
    // the complete URL.
    // Different subresource URLs are used because getRequestURLs() is called
    // for each sub test which returns a unique URL.
    if (scenario.referrer_url == "stripped-referrer") {
      promise_test(t => {
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length - 1));
        return invokeScenario(scenario)
          .then(result => checkResult(scenario.referrer_url, result))
          .finally(_ => historyBackPromise(t, scenario));
      }, "`Referer` header with length < 4k is not stripped to an origin.");

      promise_test(t => {
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length));
        return invokeScenario(scenario)
          .then(result => checkResult(scenario.referrer_url, result))
          .finally(_ => historyBackPromise(t, scenario));
      }, "`Referer` header with length == 4k is not stripped to an origin.");

      promise_test(t => {
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length + 1));
        return invokeScenario(scenario)
          .then(result => checkResult("origin", result))
          .finally(_ => historyBackPromise(t, scenario));
      }, "`Referer` header with length > 4k is stripped to an origin.");
    }

    // We test requests from inside iframes only for <img> tags.
    // This is just to preserve the previous test coverage.
    // TODO(hiroshige): Enable iframe tests for all subresource types.
    if (scenario.subresource !== "img-tag") {
      return;
    }

    // We skip <srcdoc> tests for attr-referrer, because delivering referrer
    // policy via DOM attributes inside <srcdoc> is quite similar to doing
    // so in the top-level Document.
    if (scenario.delivery_method === "attr-referrer") {
      return;
    }

    // Request in a `srcdoc` frame to ensure that it uses the referrer
    // policy of its parent,
    promise_test(_ => {
        /** @type {Array<SourceContext>} */
        const sourceContextList = [{sourceContextType: "srcdoc"}];

        return invokeScenario(scenario, sourceContextList)
          .then(result => checkResult(scenario.referrer_url, result));
      }, testDescription + " (srcdoc iframe inherits parent)");

    // We skip (top Document w/ referrer policy by HTTP headers)->
    // (<iframe srcdoc> w/ overriding referrer policy) tests, because we
    // already have similar (top Document w/ referrer policy by <meta>)->
    // (<iframe srcdoc> w/ overriding referrer policy) tests.
    if (scenario.delivery_method === "http-rp") {
      return;
    }

    // We skip (top Document w/o referrer policy)->
    // (<iframe srcdoc> w/ overriding referrer policy) tests, to simplify the
    // generator. We already have (top Document w/ referrer policy)->
    // (<iframe srcdoc> w/ overriding referrer policy) tests, which verify the
    // <iframe srcdoc>'s referrer policy behavior.
    if (scenario.referrer_policy === null) {
      return;
    }

    // Request in a `srcdoc` frame with its own referrer policy to
    // override its parent.
    promise_test(_ => {
        // Give a srcdoc iframe a referrer policy different from the
        // top-level page's policy.
        const overridingPolicy =
            scenario.referrer_policy === "no-referrer" ? "unsafe-url"
                                                       : "no-referrer";
        const overrridingExpectation =
            overridingPolicy === "no-referrer" ? "omitted"
                                               : "stripped-referrer";

        const scenarioWithoutDelivery = Object.assign({}, scenario);
        // Omit policy deliveries applied to subresource requests.
        // This is hacky method but will be removed soon.
        scenarioWithoutDelivery.delivery_method = null;

        // <iframe srcdoc> with overriding <meta> referrerPolicy.
        /** @type {Array<SourceContext>} */
        const sourceContextList = [{
            sourceContextType: "srcdoc",
            policyDeliveries: [{deliveryType: "meta",
                                key: "referrerPolicy",
                                value: overridingPolicy}]
          }];

        return invokeScenario(scenarioWithoutDelivery, sourceContextList)
          .then(result => checkResult(overrridingExpectation, result));
      }, testDescription + " (overridden by srcdoc iframe)");
  }

  return {start: runTest};
}
