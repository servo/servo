function wrapResult(server_data) {
  return {
    referrer: server_data.headers.referer,
    headers: server_data.headers
  }
}

// NOTE: This method only strips the fragment and is not in accordance to the
// recommended draft specification:
// https://w3c.github.io/webappsec/specs/referrer-policy/#null
// TODO(kristijanburnik): Implement this helper as defined by spec once added
// scenarios for URLs containing username/password/etc.
function stripUrlForUseAsReferrer(url) {
  return url.replace(/#.*$/, "");
}

function normalizePort(targetPort) {
  var defaultPorts = [80, 443];
  var isDefaultPortForProtocol = (defaultPorts.indexOf(targetPort) >= 0);

  return (targetPort == "" || isDefaultPortForProtocol) ?
          "" : ":" + targetPort;
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

  var subresourceInvoker = {
    "a-tag": requestViaAnchor,
    "area-tag": requestViaArea,
    "fetch-request": requestViaFetch,
    "iframe-tag": requestViaIframe,
    "img-tag":  requestViaImageForReferrerPolicy,
    "script-tag": requestViaScript,
    "worker-request": url => requestViaDedicatedWorker(url, {}),
    "module-worker": url => requestViaDedicatedWorker(url, {type: "module"}),
    "shared-worker": requestViaSharedWorker,
    "xhr-request": requestViaXhr
  };

  const subresourcePath = {
    "a-tag": "/referrer-policy/generic/subresource/document.py",
    "area-tag": "/referrer-policy/generic/subresource/document.py",
    "fetch-request": "/referrer-policy/generic/subresource/xhr.py",
    "iframe-tag": "/referrer-policy/generic/subresource/document.py",
    "img-tag": "/referrer-policy/generic/subresource/image.py",
    "script-tag": "/referrer-policy/generic/subresource/script.py",
    "worker-request": "/referrer-policy/generic/subresource/worker.py",
    "module-worker": "/referrer-policy/generic/subresource/worker.py",
    "shared-worker": "/referrer-policy/generic/subresource/shared-worker.py",
    "xhr-request": "/referrer-policy/generic/subresource/xhr.py"
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

      return t._scenario.target_protocol + "://" +
             domainForOrigin[t._scenario.origin] +
             normalizePort(targetPort) +
             subresourcePath[t._scenario.subresource] +
             "?redirection=" + t._scenario["redirection"] +
             "&cache_destroyer=" + (new Date()).getTime();
    },

    _constructExpectedReferrerUrl: function() {
      return referrerUrlResolver[t._scenario.referrer_url]();
    },

    // Returns a promise.
    _invokeSubresource: function(resourceRequestUrl) {
      var invoker = subresourceInvoker[t._scenario.subresource];
      // Depending on the delivery method, extend the subresource element with
      // these attributes.
      var elementAttributesForDeliveryMethod = {
        "attr-referrer":  {referrerPolicy: t._scenario.referrer_policy},
        "rel-noreferrer": {rel: "noreferrer"}
      };

      var delivery_method = t._scenario.delivery_method;

      if (delivery_method in elementAttributesForDeliveryMethod) {
        return invoker(resourceRequestUrl,
                       elementAttributesForDeliveryMethod[delivery_method],
                       t._scenario.referrer_policy);
      } else {
        return invoker(resourceRequestUrl, {}, t._scenario.referrer_policy);
      }
    },

    start: function() {
      promise_test(test => {
          const resourceRequestUrl = t._constructSubresourceUrl();
          const expectedReferrerUrl = t._constructExpectedReferrerUrl();
          return t._invokeSubresource(resourceRequestUrl)
            .then(result => {
                // Check if the result is in valid format. NOOP in release.
                sanityChecker.checkSubresourceResult(
                    test, t._scenario, resourceRequestUrl, result);

                // Check the reported URL.
                assert_equals(result.referrer,
                              expectedReferrerUrl,
                              "Reported Referrer URL is '" +
                              t._scenario.referrer_url + "'.");
                assert_equals(result.headers.referer,
                              expectedReferrerUrl,
                              "Reported Referrer URL from HTTP header is '" +
                              expectedReferrerUrl + "'");
              });
        }, t._testDescription);
    }
  }

  return t;
}
