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

  const checkResult = (expectedReferrerUrl, result) => {
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

    let currentURL = location.toString();
    const expectedReferrer =
      referrerUrlResolver[scenario.referrer_url](currentURL);

    function asyncResolve(result) {
      return new Promise((resolve, reject) => {
        step_timeout(() => resolve(result), 0);
      });
    }

    // Request in the top-level document.
    promise_test(_ => {
      return invokeRequest(subresource, [])
        .then(result => checkResult(expectedReferrer, result));
    }, testDescription);

    // `Referer` headers with length over 4k are culled down to an origin, so, let's test around
    // that boundary for tests that would otherwise return the complete URL.
    if (scenario.referrer_url == "stripped-referrer") {
      promise_test(_ => {
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length - 1));
        const expectedReferrer = location.href;
        // Ensure that we don't load the same URL as the previous test.
        subresource.url += "&-1";
        return invokeRequest(subresource, [])
          .then(result => checkResult(location.href, result))
          .then(_ => history.back())
          .then(asyncResolve);
      }, "`Referer` header with length < 4k is not stripped to an origin.");

      promise_test(_ => {
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length));
        const expectedReferrer = location.href;
        // Ensure that we don't load the same URL as the previous test.
        subresource.url += "&0";
        return invokeRequest(subresource, [])
          .then(result => checkResult(expectedReferrer, result))
          .then(_ => history.back())
          .then(asyncResolve);
      }, "`Referer` header with length == 4k is not stripped to an origin.");

      promise_test(_ => {
        const originString = referrerUrlResolver["origin"](currentURL);
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length + 1));
        // Ensure that we don't load the same URL as the previous test.
        subresource.url += "&+1";
        return invokeRequest(subresource, [])
          .then(result => checkResult(originString, result))
          .then(_ => history.back())
          .then(asyncResolve);
      }, "`Referer` header with length > 4k is stripped to an origin.");
    }

    // We test requests from inside iframes only for <img> tags.
    // This is just to preserve the previous test coverage.
    // TODO(hiroshige): Enable iframe tests for all subresource types.
    if (scenario.subresource !== "img-tag") {
      return;
    }

    // Request in a `srcdoc` frame to ensure that it uses the referrer
    // policy of its parent,
    promise_test(_ => {
        /** @type {Array<SourceContext>} */
        const sourceContextList = [{sourceContextType: "srcdoc"}];

        return invokeRequest(subresource, sourceContextList)
          .then(result => checkResult(expectedReferrer, result));
      }, testDescription + " (srcdoc iframe inherits parent)");

    // Request in a `srcdoc` frame with its own referrer policy to
    // override its parent.
    promise_test(_ => {
        // Give a srcdoc iframe a referrer policy different from the
        // top-level page's policy.
        const overridingPolicy =
            scenario.referrer_policy === "no-referrer" ? "unsafe-url"
                                                       : "no-referrer";
        const overrridingExpectedReferrer =
          referrerUrlResolver[overridingPolicy === "no-referrer"
                              ? "omitted"
                              : "stripped-referrer"](location.toString());

        /** @type {Subresource} */
        const subresourceWithoutDelivery = {
          subresourceType: scenario.subresource,
          url: urls.testUrl
        };

        // <iframe srcdoc> with overriding <meta> referrerPolicy.
        /** @type {Array<SourceContext>} */
        const sourceContextList = [{
            sourceContextType: "srcdoc",
            policyDeliveries: [{deliveryType: "meta",
                                key: "referrerPolicy",
                                value: overridingPolicy}]
          }];

        return invokeRequest(subresourceWithoutDelivery, sourceContextList)
          .then(result => checkResult(overrridingExpectedReferrer, result));
      }, testDescription + " (overridden by srcdoc iframe)");
  }

  return {start: runTest};
}
