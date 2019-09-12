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

function invokeScenario(scenario) {
  const redirectionTypeConversion = {
    "no-redirect": "no-redirect",
    "keep-scheme": "keep-scheme-redirect",
    "swap-scheme": "swap-scheme-redirect",
    "keep-origin": "keep-origin-redirect",
    "swap-origin": "swap-origin-redirect"
  };
  const subresourceTypeConversion = {
    "beacon": "beacon-request",
    "fetch": "fetch-request",
    "xhr": "xhr-request",
    "websocket": "websocket-request",
    "worker-classic": "worker-request",
    "worker-module": "module-worker",
    "worker-import-data": "module-data-worker-import",
    "sharedworker-classic": "shared-worker",
    "worklet-animation": "worklet-animation-top-level",
    "worklet-audio": "worklet-audio-top-level",
    "worklet-layout": "worklet-layout-top-level",
    "worklet-paint": "worklet-paint-top-level",
    "worklet-animation-import-data": "worklet-animation-data-import",
    "worklet-audio-import-data": "worklet-audio-data-import",
    "worklet-layout-import-data": "worklet-layout-data-import",
    "worklet-paint-import-data": "worklet-paint-data-import"
  };
  const subresourceType =
      subresourceTypeConversion[scenario.subresource] || scenario.subresource;
  const urls = getRequestURLs(
    subresourceType,
    scenario.origin,
    redirectionTypeConversion[scenario.redirection]);
  /** @type {Subresource} */
  const subresource = {
    subresourceType: subresourceType,
    url: urls.testUrl,
    policyDeliveries: scenario.subresource_policy_deliveries,
  };

  return invokeRequest(subresource, scenario.source_context_list);
}

function TestCase(scenario, testDescription, sanityChecker) {
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
    // https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer
    let referrerSource = result.sourceContextUrl;
    const sentFromSrcdoc = scenario.source_context_list.length > 0 &&
        scenario.source_context_list[scenario.source_context_list.length - 1]
        .sourceContextType === 'srcdoc';
    if (sentFromSrcdoc) {
      // Step 3. While document is an iframe srcdoc document, let document be
      // document's browsing context's browsing context container's node
      // document. [spec text]

      // Workaround for srcdoc cases. Currently we only test <iframe srcdoc>
      // inside the top-level Document, so |document| in the spec here is
      // the top-level Document.
      // This doesn't work if e.g. we test <iframe srcdoc> inside another
      // external <iframe>.
      referrerSource = location.toString();
    }
    const expectedReferrerUrl =
      referrerUrlResolver[expectation](referrerSource);

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

    promise_test(_ => {
      return invokeScenario(scenario)
        .then(result => checkResult(scenario.expectation, result));
    }, testDescription);

    // `Referer` headers with length over 4k are culled down to an origin, so,
    // let's test around that boundary for tests that would otherwise return
    // the complete URL.
    // The following tests run only on top-level Documents, because they rely
    // on navigations using `history`.
    // Different subresource URLs are used because getRequestURLs() is called
    // for each sub test which returns a unique URL.
    if (scenario.expectation == "stripped-referrer" &&
        scenario.source_context_list.length == 0) {
      promise_test(t => {
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length - 1));
        return invokeScenario(scenario)
          .then(result => checkResult(scenario.expectation, result))
          .finally(_ => historyBackPromise(t, scenario));
      }, "`Referer` header with length < 4k is not stripped to an origin.");

      promise_test(t => {
        history.pushState(null, null, "/");
        history.replaceState(null, null, "A".repeat(4096 - location.href.length));
        return invokeScenario(scenario)
          .then(result => checkResult(scenario.expectation, result))
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
  }

  return {start: runTest};
}
