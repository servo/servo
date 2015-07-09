/**
 * @fileoverview Test case for mixed-content in Web Platform Tests.
 * @author burnik@google.com (Kristijan Burnik)
 */

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
  var insecureProtocol = "http";
  var secureProtocol = "https";

  var sameOriginHost = location.hostname;
  var crossOriginHost = "{{domains[www1]}}";

  // These values can evaluate to either empty strings or a ":port" string.
  var insecurePort = getNormalizedPort(parseInt("{{ports[http][0]}}", 10));
  var securePort = getNormalizedPort(parseInt("{{ports[https][0]}}", 10));

  var resourcePath = "/mixed-content/generic/expect.py";

  // Map all endpoints to scenario for use in the test.
  var endpoint = {
    "same-origin":
      location.origin + resourcePath,
    "same-host-https":
      secureProtocol + "://" + sameOriginHost + securePort + resourcePath,
    "same-host-http":
      insecureProtocol + "://" + sameOriginHost + insecurePort + resourcePath,
    "cross-origin-https":
      secureProtocol + "://" + crossOriginHost + securePort + resourcePath,
    "cross-origin-http":
      insecureProtocol + "://" + crossOriginHost + insecurePort + resourcePath
  };

  // Mapping all the resource requesting methods to the scenario.
  var resourceMap = {
    "a-tag": requestViaAnchor,
    "area-tag": requestViaArea,
    "fetch-request": requestViaFetch,
    "form-tag": requestViaForm,
    "iframe-tag": requestViaIframe,
    "img-tag":  requestViaImage,
    "script-tag": requestViaScript,
    "worker-request": requestViaWorker,
    "xhr-request": requestViaXhr,
    "audio-tag": requestViaAudio,
    "video-tag": requestViaVideo,
    "picture-tag": requestViaPicture,
    "object-tag": requestViaObject,
    "link-css-tag": requestViaLinkStylesheet,
    "link-prefetch-tag": requestViaLinkPrefetch
  };

  sanityChecker.checkScenario(scenario, resourceMap);

  // Mapping all expected MIME types to the scenario.
  var contentType = {
    "a-tag": "text/html",
    "area-tag": "text/html",
    "fetch-request": "application/json",
    "form-tag": "text/html",
    "iframe-tag": "text/html",
    "img-tag":  "image/png",
    "script-tag": "text/javascript",
    "worker-request": "application/javascript",
    "xhr-request": "application/json",
    "audio-tag": "audio/mpeg",
    "video-tag": "video/mp4",
    "picture-tag": "image/png",
    "object-tag": "text/html",
    "link-css-tag": "text/css",
    "link-prefetch-tag": "text/html"
  };

  var mixed_content_test = async_test(description);

  function runTest() {
    var testCompleted = false;

    // Due to missing implementations, tests time out, so we fail them early.
    // TODO(kristijanburnik): Once WPT rolled in:
    //   https://github.com/w3c/testharness.js/pull/127
    // Refactor to make use of step_timeout.
    setTimeout(function() {
      mixed_content_test.step(function() {
        assert_true(testCompleted, "Expected test to complete.");
        mixed_content_test.done();
      })
    }, 1000);

    var key = guid();
    var value = guid();
    var announceResourceRequestUrl = endpoint['same-origin'] +
                                     "?action=put&key=" + key +
                                     "&value=" + value;
    var assertResourceRequestUrl = endpoint['same-origin'] +
                                  "?action=take&key=" + key;
    var resourceRequestUrl = endpoint[scenario.origin] + "?redirection=" +
                             scenario.redirection + "&action=purge&key=" +
                             key + "&content_type=" +
                             contentType[scenario.subresource];

    xhrRequest(announceResourceRequestUrl)
      .then(function(response) {
        // Send out the real resource request.
        // This should tear down the key if it's not blocked.
        return resourceMap[scenario.subresource](resourceRequestUrl);
      })
      .then(function() {
        mixed_content_test.step(function() {
          assert_equals("allowed", scenario.expectation,
                        "The triggered event should match '" +
                        scenario.expectation + "'.");
        }, "Check if success event was triggered.");

        // Send request to check if the key has been torn down.
        return xhrRequest(assertResourceRequestUrl);
      }, function(error) {
        mixed_content_test.step(function() {
          assert_equals("blocked", scenario.expectation,
                        "The triggered event should match '" +
                        scenario.expectation + "'.");
          // TODO(kristijanburnik): param "error" can be an event or error.
          // Map assertion by resource.
          // e.g.: assert_equals(e.type, "error");
        }, "Check if error event was triggered.");

        // When requestResource fails, we also check the key state.
        return xhrRequest(assertResourceRequestUrl);
      })
      .then(function(response) {
         // Now check if the value has been torn down. If it's still there,
         // we have blocked the request to mixed-content.
         mixed_content_test.step(function() {
           assert_equals(response.status, scenario.expectation,
                  "The resource request should be '" + scenario.expectation +
                  "'.");
         }, "Check if request was sent.");
         mixed_content_test.done();
         testCompleted = true;
      });

  }  // runTest

  return {start: runTest};
}  // MixedContentTestCase
