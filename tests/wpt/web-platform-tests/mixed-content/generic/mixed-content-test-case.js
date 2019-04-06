/**
 * @fileoverview Test case for mixed-content in Web Platform Tests.
 * @author burnik@google.com (Kristijan Burnik)
 */

function wrapResult(server_data) {
  // Currently the returned value is not used in mixed-content tests.
  return null;
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
  const subresourcePath = {
    "a-tag": "/common/security-features/subresource/document.py",
    "area-tag": "/common/security-features/subresource/document.py",
    "beacon-request": "/common/security-features/subresource/empty.py",
    "fetch-request": "/common/security-features/subresource/xhr.py",
    "form-tag": "/common/security-features/subresource/empty.py",
    "iframe-tag": "/common/security-features/subresource/document.py",
    "img-tag": "/common/security-features/subresource/image.py",
    "picture-tag": "/common/security-features/subresource/image.py",
    "script-tag": "/common/security-features/subresource/script.py",

    "worker-request": "/common/security-features/subresource/worker.py",
    "module-worker-top-level": "/common/security-features/subresource/worker.py",
    "module-data-worker-import": "/common/security-features/subresource/worker.py",

    "object-tag": "/common/security-features/subresource/empty.py",

    "link-css-tag": "/common/security-features/subresource/empty.py",
    "link-prefetch-tag": "/common/security-features/subresource/empty.py",
    "classic-data-worker-fetch": "/common/security-features/subresource/empty.py",

    "xhr-request": "/common/security-features/subresource/xhr.py",

    "audio-tag": "/common/security-features/subresource/audio.py",
    "video-tag": "/common/security-features/subresource/video.py",

    "websocket-request": "/stash_responder"
  };

  // Mapping all the resource requesting methods to the scenario.
  var resourceMap = {
    "a-tag": requestViaAnchor,
    "area-tag": requestViaArea,
    "beacon-request": requestViaSendBeacon,
    "fetch-request": requestViaFetch,
    "form-tag": requestViaForm,
    "iframe-tag": requestViaIframe,
    "img-tag":  requestViaImage,
    "script-tag": requestViaScript,
    "worker-request":
        url => requestViaDedicatedWorker(url),
    "module-worker-top-level":
        url => requestViaDedicatedWorker(url, {type: "module"}),
    "module-data-worker-import":
        url => requestViaDedicatedWorker(workerUrlThatImports(url), {type: "module"}),
    "classic-data-worker-fetch":
        url => requestViaDedicatedWorker(dedicatedWorkerUrlThatFetches(url), {}),
    "xhr-request": requestViaXhr,
    "audio-tag": requestViaAudio,
    "video-tag": requestViaVideo,
    "picture-tag": requestViaPicture,
    "object-tag": requestViaObject,
    "link-css-tag": requestViaLinkStylesheet,
    "link-prefetch-tag": requestViaLinkPrefetch,
    "websocket-request": requestViaWebSocket
  };

  for (const workletType of ['animation', 'audio', 'layout', 'paint']) {
    resourceMap[`worklet-${workletType}-top-level`] =
      url => requestViaWorklet(workletType, url);
    subresourcePath[`worklet-${workletType}-top-level`] =
      "/common/security-features/subresource/worker.py";

    resourceMap[`worklet-${workletType}-data-import`] =
      url => requestViaWorklet(workletType, workerUrlThatImports(url));
    subresourcePath[`worklet-${workletType}-data-import`] =
      "/common/security-features/subresource/worker.py";
  }

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

  const resourcePath = subresourcePath[scenario.subresource];

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
      wssProtocol + "://" + sameOriginHost + wssPort + resourcePath,
    "same-host-ws":
      wsProtocol + "://" + sameOriginHost + wsPort + resourcePath,
    "cross-origin-wss":
      wssProtocol + "://" + crossOriginHost + wssPort + resourcePath,
    "cross-origin-ws":
      wsProtocol + "://" + crossOriginHost + wsPort + resourcePath
  };

  sanityChecker.checkScenario(scenario, resourceMap);

  var mixed_content_test = async_test(description);

  function runTest() {
    sanityChecker.setFailTimeout(mixed_content_test);

    var key = guid();
    var value = guid();
    // We use the same path for both HTTP/S and WS/S stash requests.
    var stash_path = encodeURIComponent("/mixed-content");
    const stashEndpoint = "/common/security-features/subresource/xhr.py?key=" +
                          key + "&path=" + stash_path;
    const announceResourceRequestUrl = stashEndpoint + "&action=put&value=" +
                                       value;
    const assertResourceRequestUrl = stashEndpoint + "&action=take";
    const resourceRequestUrl = endpoint[scenario.origin] + "?redirection=" +
                             scenario.redirection + "&action=purge&key=" + key +
                             "&path=" + stash_path;

    xhrRequest(announceResourceRequestUrl)
      .then(mixed_content_test.step_func(_ => {
        // Send out the real resource request.
        // This should tear down the key if it's not blocked.
        return resourceMap[scenario.subresource](resourceRequestUrl);
      }))
      .then(mixed_content_test.step_func(_ => {
        // Send request to check if the key has been torn down.
        return xhrRequest(assertResourceRequestUrl);
      }))
      .catch(mixed_content_test.step_func(e => {
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
