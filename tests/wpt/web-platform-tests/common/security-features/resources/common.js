/**
 * @fileoverview Utilities for mixed-content in Web Platform Tests.
 * @author burnik@google.com (Kristijan Burnik)
 * Disclaimer: Some methods of other authors are annotated in the corresponding
 *     method's JSDoc.
 */

function timeoutPromise(t, ms) {
  return new Promise(resolve => { t.step_timeout(resolve, ms); });
}

/**
 * Normalizes the target port for use in a URL. For default ports, this is the
 *     empty string (omitted port), otherwise it's a colon followed by the port
 *     number. Ports 80, 443 and an empty string are regarded as default ports.
 * @param {number} targetPort The port to use
 * @return {string} The port portion for using as part of a URL.
 */
function getNormalizedPort(targetPort) {
  return ([80, 443, ""].indexOf(targetPort) >= 0) ? "" : ":" + targetPort;
}

/**
 * Creates a GUID.
 *     See: https://en.wikipedia.org/wiki/Globally_unique_identifier
 *     Original author: broofa (http://www.broofa.com/)
 *     Sourced from: http://stackoverflow.com/a/2117523/4949715
 * @return {string} A pseudo-random GUID.
 */
function guid() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

/**
 * Initiates a new XHR via GET.
 * @param {string} url The endpoint URL for the XHR.
 * @param {string} responseType Optional - how should the response be parsed.
 *     Default is "json".
 *     See: https://xhr.spec.whatwg.org/#dom-xmlhttprequest-responsetype
 * @return {Promise} A promise wrapping the success and error events.
 */
function xhrRequest(url, responseType) {
  return new Promise(function(resolve, reject) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', url, true);
    xhr.responseType = responseType || "json";

    xhr.addEventListener("error", function() {
      reject(Error("Network Error"));
    });

    xhr.addEventListener("load", function() {
      if (xhr.status != 200)
        reject(Error(xhr.statusText));
      else
        resolve(xhr.response);
    });

    xhr.send();
  });
}

/**
 * Sets attributes on a given DOM element.
 * @param {DOMElement} element The element on which to set the attributes.
 * @param {object} An object with keys (serving as attribute names) and values.
 */
function setAttributes(el, attrs) {
  attrs = attrs || {}
  for (var attr in attrs)
    el.setAttribute(attr, attrs[attr]);
}

/**
 * Binds to success and error events of an object wrapping them into a promise
 *     available through {@code element.eventPromise}. The success event
 *     resolves and error event rejects.
 * This method adds event listeners, and then removes all the added listeners
 * when one of listened event is fired.
 * @param {object} element An object supporting events on which to bind the
 *     promise.
 * @param {string} resolveEventName [="load"] The event name to bind resolve to.
 * @param {string} rejectEventName [="error"] The event name to bind reject to.
 */
function bindEvents(element, resolveEventName, rejectEventName) {
  element.eventPromise =
      bindEvents2(element, resolveEventName, element, rejectEventName);
}

// Returns a promise wrapping success and error events of objects.
// This is a variant of bindEvents that can accept separate objects for each
// events and two events to reject, and doesn't set `eventPromise`.
//
// When `resolveObject`'s `resolveEventName` event (default: "load") is
// fired, the promise is resolved with the event.
//
// When `rejectObject`'s `rejectEventName` event (default: "error") or
// `rejectObject2`'s `rejectEventName2` event (default: "error") is
// fired, the promise is rejected.
//
// `rejectObject2` is optional.
function bindEvents2(resolveObject, resolveEventName, rejectObject, rejectEventName, rejectObject2, rejectEventName2) {
  return new Promise(function(resolve, reject) {
    const actualResolveEventName = resolveEventName || "load";
    const actualRejectEventName = rejectEventName || "error";
    const actualRejectEventName2 = rejectEventName2 || "error";

    const resolveHandler = function(event) {
      cleanup();
      resolve(event);
    };

    const rejectHandler = function(event) {
      // Chromium starts propagating errors from worker.onerror to
      // window.onerror. This handles the uncaught exceptions in tests.
      event.preventDefault();
      cleanup();
      reject(event);
    };

    const cleanup = function() {
      resolveObject.removeEventListener(actualResolveEventName, resolveHandler);
      rejectObject.removeEventListener(actualRejectEventName, rejectHandler);
      if (rejectObject2) {
        rejectObject2.removeEventListener(actualRejectEventName2, rejectHandler);
      }
    };

    resolveObject.addEventListener(actualResolveEventName, resolveHandler);
    rejectObject.addEventListener(actualRejectEventName, rejectHandler);
    if (rejectObject2) {
      rejectObject2.addEventListener(actualRejectEventName2, rejectHandler);
    }
  });
}

/**
 * Creates a new DOM element.
 * @param {string} tagName The type of the DOM element.
 * @param {object} attrs A JSON with attributes to apply to the element.
 * @param {DOMElement} parent Optional - an existing DOM element to append to
 *     If not provided, the returned element will remain orphaned.
 * @param {boolean} doBindEvents Optional - Whether to bind to load and error
 *     events and provide the promise wrapping the events via the element's
 *     {@code eventPromise} property. Default value evaluates to false.
 * @return {DOMElement} The newly created DOM element.
 */
function createElement(tagName, attrs, parentNode, doBindEvents) {
  var element = document.createElement(tagName);

  if (doBindEvents)
    bindEvents(element);

  // We set the attributes after binding to events to catch any
  // event-triggering attribute changes. E.g. form submission.
  //
  // But be careful with images: unlike other elements they will start the load
  // as soon as the attr is set, even if not in the document yet, and sometimes
  // complete it synchronously, so the append doesn't have the effect we want.
  // So for images, we want to set the attrs after appending, whereas for other
  // elements we want to do it before appending.
  var isImg = (tagName == "img");
  if (!isImg)
    setAttributes(element, attrs);

  if (parentNode)
    parentNode.appendChild(element);

  if (isImg)
    setAttributes(element, attrs);

  return element;
}

function createRequestViaElement(tagName, attrs, parentNode) {
  return createElement(tagName, attrs, parentNode, true).eventPromise;
}

/**
 * Creates a new empty iframe and appends it to {@code document.body} .
 * @param {string} name The name and ID of the new iframe.
 * @param {boolean} doBindEvents Whether to bind load and error events.
 * @return {DOMElement} The newly created iframe.
 */
function createHelperIframe(name, doBindEvents) {
  return createElement("iframe",
                       {"name": name, "id": name},
                       document.body,
                       doBindEvents);
}

/**
 * requestVia*() functions return promises that are resolved on successful
 * requests with objects of the same "type", i.e. objects that contains
 * the same sets of keys that are fixed within one category of tests (e.g.
 * within wpt/referrer-policy tests).
 * wrapResult() (that should be defined outside this file) is used to convert
 * the response bodies of subresources into the expected result objects in some
 * cases, and in other cases the result objects are constructed more directly.
 * TODO(https://crbug.com/906850): Clean up the semantics around this, e.g.
 * use (or not use) wrapResult() consistently, unify the arguments, etc.
 */

/**
 * Creates a new iframe, binds load and error events, sets the src attribute and
 *     appends it to {@code document.body} .
 * @param {string} url The src for the iframe.
 * @return {Promise} The promise for success/error events.
 */
function requestViaIframe(url, additionalAttributes) {
  const iframe = createElement(
      "iframe",
      Object.assign({"src": url}, additionalAttributes),
      document.body,
      false);
  return bindEvents2(window, "message", iframe, "error", window, "error")
      .then(event => {
          assert_equals(event.source, iframe.contentWindow);
          return event.data;
        });
}

/**
 * Creates a new image, binds load and error events, sets the src attribute and
 *     appends it to {@code document.body} .
 * @param {string} url The src for the image.
 * @return {Promise} The promise for success/error events.
 */
function requestViaImage(url) {
  return createRequestViaElement("img", {"src": url}, document.body);
}

// Helpers for requestViaImageForReferrerPolicy().
function loadImageInWindow(src, attributes, w) {
  return new Promise((resolve, reject) => {
    var image = new w.Image();
    image.crossOrigin = "Anonymous";
    image.onload = function() {
      resolve(image);
    };

    // Extend element with attributes. (E.g. "referrerPolicy" or "rel")
    if (attributes) {
      for (var attr in attributes) {
        image[attr] = attributes[attr];
      }
    }

    image.src = src;
    w.document.body.appendChild(image)
  });
}

function extractImageData(img) {
    var canvas = document.createElement("canvas");
    var context = canvas.getContext('2d');
    context.drawImage(img, 0, 0);
    var imgData = context.getImageData(0, 0, img.clientWidth, img.clientHeight);
    return imgData.data;
}

function decodeImageData(rgba) {
  var rgb = new Uint8ClampedArray(rgba.length);

  // RGBA -> RGB.
  var rgb_length = 0;
  for (var i = 0; i < rgba.length; ++i) {
    // Skip alpha component.
    if (i % 4 == 3)
      continue;

    // Zero is the string terminator.
    if (rgba[i] == 0)
      break;

    rgb[rgb_length++] = rgba[i];
  }

  // Remove trailing nulls from data.
  rgb = rgb.subarray(0, rgb_length);
  var string_data = (new TextDecoder("ascii")).decode(rgb);

  return JSON.parse(string_data);
}

// A variant of requestViaImage for referrer policy tests.
// This tests many patterns of <iframe>s to test referrer policy inheritance.
// TODO(https://crbug.com/906850): Merge this into requestViaImage().
// <iframe>-related code should be moved outside requestViaImage*().
function requestViaImageForReferrerPolicy(url, attributes, referrerPolicy) {
  // For images, we'll test:
  // - images in a `srcdoc` frame to ensure that it uses the referrer
  //   policy of its parent,
  // - images in a top-level document,
  // - and images in a `srcdoc` frame with its own referrer policy to
  //   override its parent.

  var iframeWithoutOwnPolicy = document.createElement('iframe');
  var noSrcDocPolicy = new Promise((resolve, reject) => {
        iframeWithoutOwnPolicy.srcdoc = "Hello, world.";
        iframeWithoutOwnPolicy.onload = resolve;
        document.body.appendChild(iframeWithoutOwnPolicy);
      })
    .then(() => {
        var nextUrl = url + "&cache_destroyer2=" + (new Date()).getTime();
        return loadImageInWindow(nextUrl, attributes,
                                 iframeWithoutOwnPolicy.contentWindow);
      })
    .then(function (img) {
        return decodeImageData(extractImageData(img));
      });

  // Give a srcdoc iframe a referrer policy different from the top-level page's policy.
  var iframePolicy = (referrerPolicy === "no-referrer") ? "unsafe-url" : "no-referrer";
  var iframeWithOwnPolicy = document.createElement('iframe');
  var srcDocPolicy = new Promise((resolve, reject) => {
        iframeWithOwnPolicy.srcdoc = "<meta name='referrer' content='" + iframePolicy + "'>Hello world.";
        iframeWithOwnPolicy.onload = resolve;
        document.body.appendChild(iframeWithOwnPolicy);
      })
    .then(() => {
        var nextUrl = url + "&cache_destroyer3=" + (new Date()).getTime();
        return loadImageInWindow(nextUrl, null,
                                 iframeWithOwnPolicy.contentWindow);
      })
    .then(function (img) {
        return decodeImageData(extractImageData(img));
      });

  var pagePolicy = loadImageInWindow(url, attributes, window)
    .then(function (img) {
        return decodeImageData(extractImageData(img));
      });

  return Promise.all([noSrcDocPolicy, srcDocPolicy, pagePolicy]).then(values => {
    assert_equals(values[0].headers.referer, values[2].headers.referer, "Referrer inside 'srcdoc' without its own policy should be the same as embedder's referrer.");
    assert_equals((iframePolicy === "no-referrer" ? undefined : document.location.href), values[1].headers.referer, "Referrer inside 'srcdoc' should use the iframe's policy if it has one");
    return wrapResult(values[2]);
  });
}

/**
 * Initiates a new XHR GET request to provided URL.
 * @param {string} url The endpoint URL for the XHR.
 * @return {Promise} The promise for success/error events.
 */
function requestViaXhr(url) {
  return xhrRequest(url).then(result => wrapResult(result));
}

/**
 * Initiates a new GET request to provided URL via the Fetch API.
 * @param {string} url The endpoint URL for the Fetch.
 * @return {Promise} The promise for success/error events.
 */
function requestViaFetch(url) {
  return fetch(url)
    .then(res => res.json())
    .then(j => wrapResult(j));
}

function dedicatedWorkerUrlThatFetches(url) {
  return `data:text/javascript,
    fetch('${url}')
      .then(() => postMessage(''),
            () => postMessage(''));`;
}

function workerUrlThatImports(url) {
  return `data:text/javascript,import '${url}';`;
}

/**
 * Creates a new Worker, binds message and error events wrapping them into.
 *     {@code worker.eventPromise} and posts an empty string message to start
 *     the worker.
 * @param {string} url The endpoint URL for the worker script.
 * @param {object} options The options for Worker constructor.
 * @return {Promise} The promise for success/error events.
 */
function requestViaDedicatedWorker(url, options) {
  var worker;
  try {
    worker = new Worker(url, options);
  } catch (e) {
    return Promise.reject(e);
  }
  worker.postMessage('');
  return bindEvents2(worker, "message", worker, "error")
    .then(event => wrapResult(event.data));
}

function requestViaSharedWorker(url) {
  var worker;
  try {
    worker = new SharedWorker(url);
  } catch(e) {
    return Promise.reject(e);
  }
  const promise = bindEvents2(worker.port, "message", worker, "error")
    .then(event => wrapResult(event.data));
  worker.port.start();
  return promise;
}

// Returns a reference to a worklet object corresponding to a given type.
function get_worklet(type) {
  if (type == 'animation')
    return CSS.animationWorklet;
  if (type == 'layout')
    return CSS.layoutWorklet;
  if (type == 'paint')
    return CSS.paintWorklet;
  if (type == 'audio')
    return new OfflineAudioContext(2,44100*40,44100).audioWorklet;

  assert_unreached('unknown worklet type is passed.');
  return undefined;
}

function requestViaWorklet(type, url) {
  try {
    return get_worklet(type).addModule(url);
  } catch (e) {
    return Promise.reject(e);
  }
}

/**
 * Sets the href attribute on a navigable DOM element and performs a navigation
 *     by clicking it. To avoid navigating away from the current execution
 *     context, a target attribute is set to point to a new helper iframe.
 * @param {DOMElement} navigableElement The navigable DOMElement
 * @param {string} url The href for the navigable element.
 * @return {Promise} The promise for success/error events.
 */
function requestViaNavigable(navigableElement, url) {
  var iframe = createHelperIframe(guid(), false);
  setAttributes(navigableElement,
                {"href": url,
                 "target": iframe.name});

  const promise =
    bindEvents2(window, "message", iframe, "error", window, "error")
      .then(event => {
          assert_equals(event.source, iframe.contentWindow, "event.source");
          return event.data;
        });
  navigableElement.click();
  return promise;
}

/**
 * Creates a new anchor element, appends it to {@code document.body} and
 *     performs the navigation.
 * @param {string} url The URL to navigate to.
 * @return {Promise} The promise for success/error events.
 */
function requestViaAnchor(url, additionalAttributes) {
  var a = createElement(
      "a",
      Object.assign({"innerHTML": "Link to resource"}, additionalAttributes),
      document.body);

  return requestViaNavigable(a, url);
}

/**
 * Creates a new area element, appends it to {@code document.body} and performs
 *     the navigation.
 * @param {string} url The URL to navigate to.
 * @return {Promise} The promise for success/error events.
 */
function requestViaArea(url, additionalAttributes) {
  var area = createElement(
      "area",
      Object.assign({}, additionalAttributes),
      document.body);

  // TODO(kristijanburnik): Append to map and add image.
  return requestViaNavigable(area, url);
}

/**
 * Creates a new script element, sets the src to url, and appends it to
 *     {@code document.body}.
 * @param {string} url The src URL.
 * @return {Promise} The promise for success/error events.
 */
function requestViaScript(url, additionalAttributes) {
  const script = createElement(
      "script",
      Object.assign({"src": url}, additionalAttributes),
      document.body,
      false);

  return bindEvents2(window, "message", script, "error", window, "error")
    .then(event => wrapResult(event.data));
}

/**
 * Creates a new form element, sets attributes, appends it to
 *     {@code document.body} and submits the form.
 * @param {string} url The URL to submit to.
 * @return {Promise} The promise for success/error events.
 */
function requestViaForm(url) {
  var iframe = createHelperIframe(guid());
  var form = createElement("form",
                           {"action": url,
                            "method": "POST",
                            "target": iframe.name},
                           document.body);
  bindEvents(iframe);
  form.submit();

  return iframe.eventPromise;
}

/**
 * Creates a new link element for a stylesheet, binds load and error events,
 *     sets the href to url and appends it to {@code document.head}.
 * @param {string} url The URL for a stylesheet.
 * @return {Promise} The promise for success/error events.
 */
function requestViaLinkStylesheet(url) {
  return createRequestViaElement("link",
                                 {"rel": "stylesheet", "href": url},
                                 document.head);
}

/**
 * Creates a new link element for a prefetch, binds load and error events, sets
 *     the href to url and appends it to {@code document.head}.
 * @param {string} url The URL of a resource to prefetch.
 * @return {Promise} The promise for success/error events.
 */
function requestViaLinkPrefetch(url) {
  var link = document.createElement('link');
  if (link.relList && link.relList.supports && link.relList.supports("prefetch")) {
    return createRequestViaElement("link",
                                   {"rel": "prefetch", "href": url},
                                   document.head);
  } else {
    return Promise.reject("This browser does not support 'prefetch'.");
  }
}

/**
 * Initiates a new beacon request.
 * @param {string} url The URL of a resource to prefetch.
 * @return {Promise} The promise for success/error events.
 */
async function requestViaSendBeacon(url) {
  function wait(ms) {
    return new Promise(resolve => step_timeout(resolve, ms));
  }
  if (!navigator.sendBeacon(url)) {
    // If mixed-content check fails, it should return false.
    throw new Error('sendBeacon() fails.');
  }
  // We don't have a means to see the result of sendBeacon() request
  // for sure. Let's wait for a while and let the generic test function
  // ask the server for the result.
  await wait(500);
  return 'allowed';
}

/**
 * Creates a new media element with a child source element, binds loadeddata and
 *     error events, sets attributes and appends to document.body.
 * @param {string} type The type of the media element (audio/video/picture).
 * @param {object} media_attrs The attributes for the media element.
 * @param {object} source_attrs The attributes for the child source element.
 * @return {DOMElement} The newly created media element.
 */
function createMediaElement(type, media_attrs, source_attrs) {
  var mediaElement = createElement(type, {});

  var sourceElement = createElement("source", {});

  mediaElement.eventPromise = new Promise(function(resolve, reject) {
    mediaElement.addEventListener("loadeddata", function (e) {
      resolve(e);
    });

    // Safari doesn't fire an `error` event when blocking mixed content.
    mediaElement.addEventListener("stalled", function(e) {
      reject(e);
    });

    sourceElement.addEventListener("error", function(e) {
      reject(e);
    });
  });

  setAttributes(mediaElement, media_attrs);
  setAttributes(sourceElement, source_attrs);

  mediaElement.appendChild(sourceElement);
  document.body.appendChild(mediaElement);

  return mediaElement;
}

/**
 * Creates a new video element, binds loadeddata and error events, sets
 *     attributes and source URL and appends to {@code document.body}.
 * @param {string} url The URL of the video.
 * @return {Promise} The promise for success/error events.
 */
function requestViaVideo(url) {
  return createMediaElement("video",
                            {},
                            {"src": url}).eventPromise;
}

/**
 * Creates a new audio element, binds loadeddata and error events, sets
 *     attributes and source URL and appends to {@code document.body}.
 * @param {string} url The URL of the audio.
 * @return {Promise} The promise for success/error events.
 */
function requestViaAudio(url) {
  return createMediaElement("audio",
                            {},
                            {"type": "audio/wav", "src": url}).eventPromise;
}

/**
 * Creates a new picture element, binds loadeddata and error events, sets
 *     attributes and source URL and appends to {@code document.body}. Also
 *     creates new image element appending it to the picture
 * @param {string} url The URL of the image for the source and image elements.
 * @return {Promise} The promise for success/error events.
 */
function requestViaPicture(url) {
  var picture = createMediaElement("picture", {}, {"srcset": url,
                                                "type": "image/png"});
  return createRequestViaElement("img", {"src": url}, picture);
}

/**
 * Creates a new object element, binds load and error events, sets the data to
 *     url, and appends it to {@code document.body}.
 * @param {string} url The data URL.
 * @return {Promise} The promise for success/error events.
 */
function requestViaObject(url) {
  return createRequestViaElement("object", {"data": url, "type": "text/html"}, document.body);
}

/**
 * Creates a new WebSocket pointing to {@code url} and sends a message string
 * "echo". The {@code message} and {@code error} events are triggering the
 * returned promise resolve/reject events.
 * @param {string} url The URL for WebSocket to connect to.
 * @return {Promise} The promise for success/error events.
 */
function requestViaWebSocket(url) {
  return new Promise(function(resolve, reject) {
    var websocket = new WebSocket(url);

    websocket.addEventListener("message", function(e) {
      resolve(e.data);
    });

    websocket.addEventListener("open", function(e) {
      websocket.send("echo");
    });

    websocket.addEventListener("error", function(e) {
      reject(e)
    });
  })
  .then(data => {
      return JSON.parse(data);
    });
}

// SanityChecker does nothing in release mode. See sanity-checker.js for debug
// mode.
function SanityChecker() {}
SanityChecker.prototype.checkScenario = function() {};
SanityChecker.prototype.setFailTimeout = function(test, timeout) {};
SanityChecker.prototype.checkSubresourceResult = function() {};
