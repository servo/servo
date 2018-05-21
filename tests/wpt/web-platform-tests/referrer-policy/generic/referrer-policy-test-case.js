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

  var subresourceInvoker = {
    "a-tag": queryLink,
    "area-tag": queryAreaLink,
    "fetch-request": queryFetch,
    "iframe-tag": queryIframe,
    "img-tag":  queryImage,
    "script-tag": queryScript,
    "worker-request": queryWorker,
    "xhr-request": queryXhr
  };

  var referrerUrlResolver = {
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

  var t = {
    _scenario: scenario,
    _testDescription: testDescription,
    _subresourceUrl: null,
    _expectedReferrerUrl: null,
    _constructSubresourceUrl: function() {
      // TODO(kristijanburnik): We should assert that these two domains are
      // different. E.g. If someone runs the tets over www, this would fail.
      var domainForOrigin = {
        "cross-origin":"{{domains[www1]}}",
        "same-origin": location.hostname
      };

      // Values obtained and replaced by the wptserve pipeline:
      // http://wptserve.readthedocs.org/en/latest/pipes.html#built-in-pipes
      var portForProtocol = {
        "http": parseInt("{{ports[http][0]}}"),
        "https": parseInt("{{ports[https][0]}}")
      }

      var targetPort = portForProtocol[t._scenario.target_protocol];

      t._subresourceUrl = t._scenario.target_protocol + "://" +
                          domainForOrigin[t._scenario.origin] +
                          normalizePort(targetPort) +
                          t._scenario["subresource_path"] +
                          "?redirection=" + t._scenario["redirection"] +
                          "&cache_destroyer=" + (new Date()).getTime();
    },

    _constructExpectedReferrerUrl: function() {
      t._expectedReferrerUrl = referrerUrlResolver[t._scenario.referrer_url]();
    },

    _invokeSubresource: function(callback, test) {
      var invoker = subresourceInvoker[t._scenario.subresource];

      // Depending on the delivery method, extend the subresource element with
      // these attributes.
      var elementAttributesForDeliveryMethod = {
        "attr-referrer":  {referrerPolicy: t._scenario.referrer_policy},
        "rel-noreferrer": {rel: "noreferrer"}
      };

      var delivery_method = t._scenario.delivery_method;

      if (delivery_method in elementAttributesForDeliveryMethod) {
        invoker(t._subresourceUrl,
                callback,
                elementAttributesForDeliveryMethod[delivery_method],
                t._scenario.referrer_policy,
                test);
      } else {
        invoker(t._subresourceUrl, callback, null, t._scenario.referrer_policy, test);
      }

    },

    start: function() {
      t._constructSubresourceUrl();
      t._constructExpectedReferrerUrl();

      var test = async_test(t._testDescription);

      t._invokeSubresource(function(result) {
        // Check if the result is in valid format. NOOP in release.
        sanityChecker.checkSubresourceResult(
            test, t._scenario, t._subresourceUrl, result);

        // Check the reported URL.
        test.step(function() {
          assert_equals(result.referrer,
                        t._expectedReferrerUrl,
                        "Reported Referrer URL is '" +
                        t._scenario.referrer_url + "'.");
          assert_equals(result.headers.referer,
                        t._expectedReferrerUrl,
                        "Reported Referrer URL from HTTP header is '" +
                        t._expectedReferrerUrl + "'");
        }, "Reported Referrer URL is as expected: " + t._scenario.referrer_url);

        test.done();
      }, test);

    }
  }

  return t;
}
