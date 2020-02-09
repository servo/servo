// NOTE: This method only strips the fragment and is not in accordance to the
// recommended draft specification:
// https://w3c.github.io/webappsec/specs/referrer-policy/#null
// TODO(kristijanburnik): Implement this helper as defined by spec once added
// scenarios for URLs containing username/password/etc.
function stripUrlForUseAsReferrer(url) {
  return url.replace(/#.*$/, "");
}

function invokeScenario(scenario) {
  const urls = getRequestURLs(
    scenario.subresource,
    scenario.origin,
    scenario.redirection);
  /** @type {Subresource} */
  const subresource = {
    subresourceType: scenario.subresource,
    url: urls.testUrl,
    policyDeliveries: scenario.subresource_policy_deliveries,
  };

  return invokeRequest(subresource, scenario.source_context_list);
}

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

function checkResult(scenario, expectation, result) {
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
}

function runLengthTest(scenario, urlLength, expectation, testDescription) {
  // `Referer` headers with length over 4k are culled down to an origin, so,
  // let's test around that boundary for tests that would otherwise return
  // the complete URL.
  history.pushState(null, null, "/");
  history.replaceState(null, null,
      "A".repeat(urlLength - location.href.length));

  promise_test(t => {
    assert_equals(scenario.expectation, "stripped-referrer");
    // Only on top-level Window, due to navigations using `history`.
    assert_equals(scenario.source_context_list.length, 0);

    return invokeScenario(scenario)
      .then(result => checkResult(scenario, expectation, result));
  }, testDescription);
}

function TestCase(scenario, testDescription, sanityChecker) {
  // This check is A NOOP in release.
  sanityChecker.checkScenario(scenario);

  function runTest() {
    promise_test(_ => {
      return invokeScenario(scenario)
        .then(result => checkResult(scenario, scenario.expectation, result));
    }, testDescription);
  }

  return {start: runTest};
}
