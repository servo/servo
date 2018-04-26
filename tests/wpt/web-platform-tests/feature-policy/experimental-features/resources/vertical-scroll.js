const url_base = "/feature-policy/experimental-features/resources/";
window.messageResponseCallback = null;

function rectMaxY(rect) {
  return rect.height + rect.y;
}

function rectMaxX(rect) {
  return rect.width + rect.x;
}

function isEmptyRect(rect) {
  return !rect.width || !rect.height;
}

// Returns true if the given rectangles intersect.
function rects_intersect(rect1, rect2) {
  if (isEmptyRect(rect1) || isEmptyRect(rect2))
    return false;
  return rect1.x < rectMaxX(rect2) &&
         rect2.x < rectMaxX(rect1) &&
         rect1.y < rectMaxY(rect2) &&
         rect2.y < rectMaxY(rect1);
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

function rectToString(rect) {
  return `Location: (${rect.x}, ${rect.y}) Size: (${rect.width}, ${rect.height})`;
}

function onMessage(e) {
  if (window.messageResponseCallback) {
    window.messageResponseCallback(e.data);
    window.messageResponseCallback = null;
  }
}

window.addEventListener("message", onMessage);
