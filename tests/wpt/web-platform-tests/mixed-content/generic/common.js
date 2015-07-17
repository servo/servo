/**
 * @fileoverview Utilities for mixed-content in Web Platform Tests.
 * @author burnik@google.com (Kristijan Burnik)
 * Disclaimer: Some methods of other authors are annotated in the corresponding
 *     method's JSDoc.
 */

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
        return reject(Error(xhr.statusText));

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
 * @param {object} element An object supporting events on which to bind the
 *     promise.
 * @param {string} resolveEventName [="load"] The event name to bind resolve to.
 * @param {string} rejectEventName [="error"] The event name to bind reject to.
 */
function bindEvents(element, resolveEventName, rejectEventName) {
  element.eventPromise = new Promise(function(resolve, reject) {
    element.addEventListener(resolveEventName  || "load", resolve);
    element.addEventListener(rejectEventName || "error", reject);
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
function createElement(tagName, attrs, parent, doBindEvents) {
  var element = document.createElement(tagName);

  if (doBindEvents)
    bindEvents(element);

  // We set the attributes after binding to events to catch any
  // event-triggering attribute changes. E.g. form submission.
  setAttributes(element, attrs);

  if (parent)
    parent.appendChild(element);

  return element;
}

function createRequestViaElement(tagName, attrs, parent) {
  return createElement(tagName, attrs, parent, true).eventPromise;
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
 * Creates a new iframe, binds load and error events, sets the src attribute and
 *     appends it to {@code document.body} .
 * @param {string} url The src for the iframe.
 * @return {Promise} The promise for success/error events.
 */
function requestViaIframe(url) {
  return createRequestViaElement("iframe", {"src": url}, document.body);
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

/**
 * Initiates a new XHR GET request to provided URL.
 * @param {string} url The endpoint URL for the XHR.
 * @return {Promise} The promise for success/error events.
 */
function requestViaXhr(url) {
  return xhrRequest(url);
}

/**
 * Initiates a new GET request to provided URL via the Fetch API.
 * @param {string} url The endpoint URL for the Fetch.
 * @return {Promise} The promise for success/error events.
 */
function requestViaFetch(url) {
  return fetch(url);
}

/**
 * Creates a new Worker, binds message and error events wrapping them into.
 *     {@code worker.eventPromise} and posts an empty string message to start
 *     the worker.
 * @param {string} url The endpoint URL for the worker script.
 * @return {Promise} The promise for success/error events.
 */
function requestViaWorker(url) {
  var worker = new Worker(url);
  bindEvents(worker, "message", "error");
  worker.postMessage('');

  return worker.eventPromise;
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
  var iframe = createHelperIframe(guid(), true);
  setAttributes(navigableElement,
                {"href": url,
                 "target": iframe.name});
  navigableElement.click();

  return iframe.eventPromise;
}

/**
 * Creates a new anchor element, appends it to {@code document.body} and
 *     performs the navigation.
 * @param {string} url The URL to navigate to.
 * @return {Promise} The promise for success/error events.
 */
function requestViaAnchor(url) {
  var a = createElement("a", {"innerHTML": "Link to resource"}, document.body);

  return requestViaNavigable(a, url);
}

/**
 * Creates a new area element, appends it to {@code document.body} and performs
 *     the navigation.
 * @param {string} url The URL to navigate to.
 * @return {Promise} The promise for success/error events.
 */
function requestViaArea(url) {
  var area = createElement("area", {}, document.body);

  return requestViaNavigable(area, url);
}

/**
 * Creates a new script element, sets the src to url, and appends it to
 *     {@code document.body}.
 * @param {string} url The src URL.
 * @return {Promise} The promise for success/error events.
 */
function requestViaScript(url) {
  return createRequestViaElement("script", {"src": url}, document.body);
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
  // TODO(kristijanburnik): Check if prefetch should support load and error
  // events. For now we assume it's not specified.
  // https://developer.mozilla.org/en-US/docs/Web/HTTP/Link_prefetching_FAQ
  return createRequestViaElement("link",
                                 {"rel": "prefetch", "href": url},
                                 document.head);
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
  var sourceElement = createElement("source", {}, mediaElement);

  mediaElement.eventPromise = new Promise(function(resolve, reject) {
    mediaElement.addEventListener("loadeddata", resolve);
    // Notice that the source element will raise the error.
    sourceElement.addEventListener("error", reject);
  });

  setAttributes(mediaElement, media_attrs);
  setAttributes(sourceElement, source_attrs);
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
                            {type: "video/mp4", src: url}).eventPromise;
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
                            {type: "audio/mpeg", src: url}).eventPromise;
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
  return createRequestViaElement("object", {"data": url}, document.body);
}

// SanityChecker does nothing in release mode. See sanity-checker.js for debug
// mode.
function SanityChecker() {}
SanityChecker.prototype.checkScenario = function() {};
