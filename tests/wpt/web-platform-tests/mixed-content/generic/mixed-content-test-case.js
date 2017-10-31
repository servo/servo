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
  var httpProtocol = "http";
  var httpsProtocol = "https";
  var wsProtocol = "ws";
  var wssProtocol = "wss";

  var sameOriginHost = location.hostname;
  var crossOriginHost = "{{domains[www1]}}";

  // These values can evaluate to either empty strings or a ":port" string.
  var httpPort = getNormalizedPort(parseInt("{{ports[http][0]}}", 10));
  var httpsPort = getNormalizedPort(parseInt("{{ports[https][0]}}", 10));
  var wsPort = getNormalizedPort(parseInt("{{ports[ws][0]}}", 10));
  var wssPort = getNormalizedPort(parseInt("{{ports[wss][0]}}", 10));

  var resourcePath = "/mixed-content/generic/expect.py";
  var wsResourcePath = "/stash_responder";

  // Map all endpoints to scenario for use in the test.
  var endpoint = {
    "same-origin":
      location.origin + resourcePath,
    "same-host-https":
      httpsProtocol + "://" + sameOriginHost + httpsPort + resourcePath,
    "same-host-http":
      httpProtocol + "://" + sameOriginHost + httpPort + resourcePath,
    "cross-origin-https":
      httpsProtocol + "://" + crossOriginHost + httpsPort + resourcePath,
    "cross-origin-http":
      httpProtocol + "://" + crossOriginHost + httpPort + resourcePath,
    "same-host-wss":
      wssProtocol + "://" + sameOriginHost + wssPort + wsResourcePath,
    "same-host-ws":
      wsProtocol + "://" + sameOriginHost + wsPort + wsResourcePath,
    "cross-origin-wss":
      wssProtocol + "://" + crossOriginHost + wssPort + wsResourcePath,
    "cross-origin-ws":
      wsProtocol + "://" + crossOriginHost + wsPort + wsResourcePath
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
    "link-prefetch-tag": requestViaLinkPrefetch,
    "websocket-request": requestViaWebSocket
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
    "audio-tag": "audio/wav",
    "video-tag": "video/ogg",
    "picture-tag": "image/png",
    "object-tag": "text/html",
    "link-css-tag": "text/css",
    "link-prefetch-tag": "text/html",
    "websocket-request": "application/json"
  };

  var mixed_content_test = async_test(description);

  function runTest() {
    sanityChecker.setFailTimeout(mixed_content_test);

    var key = guid();
    var value = guid();
    // We use the same path for both HTTP/S and WS/S stash requests.
    var stash_path = encodeURIComponent("/mixed-content");
    var announceResourceRequestUrl = endpoint['same-origin'] +
                                     "?action=put&key=" + key +
                                     "&value=" + value +
                                     "&path=" + stash_path;
    var assertResourceRequestUrl = endpoint['same-origin'] +
                                  "?action=take&key=" + key +
                                  "&path=" + stash_path;
    var resourceRequestUrl = endpoint[scenario.origin] + "?redirection=" +
                             scenario.redirection + "&action=purge&key=" + key +
                             "&path=" + stash_path + "&content_type=" +
                             contentType[scenario.subresource];

    xhrRequest(announceResourceRequestUrl)
      .then(mixed_content_test.step_func(_ => {
        // Send out the real resource request.
        // This should tear down the key if it's not blocked.
        return resourceMap[scenario.subresource](resourceRequestUrl);
      }))
      .then(mixed_content_test.step_func(_ => {
        mixed_content_test.step(function() {
          assert_equals("allowed", scenario.expectation,
                        "The triggered event should match '" +
                        scenario.expectation + "'.");
        }, "Check if success event was triggered.");

        // Send request to check if the key has been torn down.
        return xhrRequest(assertResourceRequestUrl);
      }))
      .catch(mixed_content_test.step_func(e => {
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
      }))
      .then(mixed_content_test.step_func_done(response => {
         // Now check if the value has been torn down. If it's still there,
         // we have blocked the request to mixed-content.
         assert_equals(response.status, scenario.expectation,
           "The resource request should be '" + scenario.expectation + "'.");
      }));

  }  // runTest

  return {start: mixed_content_test.step_func(runTest) };
}  // MixedContentTestCase
