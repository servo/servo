const url_base = "/feature-policy/experimental-features/resources/";
window.messageResponseCallback = null;

function setFeatureState(iframe, feature, origins) {
    iframe.setAttribute("allow", `${feature} ${origins};`);
}

// Returns a promise which is resolved when the <iframe> is navigated to |url|
// and "load" handler has been called.
function loadUrlInIframe(iframe, url) {
  return new Promise((resolve) => {
    iframe.addEventListener("load", resolve);
    iframe.src = url;
  });
}

// Posts |message| to |target| and resolves the promise with the response coming
// back from |target|.
function sendMessageAndGetResponse(target, message) {
  return new Promise((resolve) => {
    window.messageResponseCallback = resolve;
    target.postMessage(message, "*");
  });
}


function onMessage(e) {
  if (window.messageResponseCallback) {
    window.messageResponseCallback(e.data);
    window.messageResponseCallback = null;
  }
}

window.addEventListener("message", onMessage);

// Waits for |load_timeout| before resolving the promise. It will resolve the
// promise sooner if a message event with |e.data.id| of |id| is received.
// In such a case the response is the contents of the message |e.data.contents|.
// Otherwise, returns false (when timeout occurs).
function waitForMessageOrTimeout(t, id, load_timeout) {
  return new Promise((resolve) => {
      window.addEventListener(
        "message",
        (e) => {
          if (!e.data || e.data.id !== id)
            return;
          resolve(e.data.contents);
        }
      );
      t.step_timeout(() => { resolve(false); }, load_timeout);
  });
}

function createIframe(container, attributes) {
  var new_iframe = document.createElement("iframe");
  for (attr_name in attributes)
    new_iframe.setAttribute(attr_name, attributes[attr_name]);
  container.appendChild(new_iframe);
  return new_iframe;
}
