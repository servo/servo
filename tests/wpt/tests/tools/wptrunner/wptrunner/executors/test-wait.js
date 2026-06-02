const initialized = !!window.__wptrunner_url;
window.__wptrunner_testdriver_callback = arguments[arguments.length - 1];
window.__wptrunner_url = arguments.length > 1 ? arguments[0] : location.href;
var observer = null;
var root = document.documentElement;

function wait_load() {
  if (Document.prototype.hasOwnProperty("fonts")) {
    document.fonts.ready.then(wait_paints);
  } else {
    // This might take the screenshot too early, depending on whether the
    // load event is blocked on fonts being loaded. See:
    // https://github.com/w3c/csswg-drafts/issues/1088
    wait_paints();
  }
}

function wait_paints() {
  // As of 2017-04-05, the Chromium web browser exhibits a rendering bug
  // (https://bugs.chromium.org/p/chromium/issues/detail?id=708757) that
  // produces instability during screen capture. The following use of
  // `requestAnimationFrame` is intended as a short-term workaround, though
  // it is not guaranteed to resolve the issue.
  //
  // For further detail, see:
  // https://github.com/jugglinmike/chrome-screenshot-race/issues/1

  requestAnimationFrame(function() {
    requestAnimationFrame(function() {
      screenshot_if_ready();
    });
  });
}

function screenshot_if_ready() {
  if (root && root.classList.contains("%(classname)s")) {
    if (observer === null) {
      observer = new MutationObserver(wait_paints);
      observer.observe(root, {attributes: true});
      var event = new Event("TestRendered", {bubbles: true});
      root.dispatchEvent(event);
    }
    return;
  }
  if (observer !== null) {
    observer.disconnect();
  }
  if (window.__wptrunner_message_queue) {
    __wptrunner_message_queue.push({type: "complete"});
  } else {
    // Not using `testdriver.js`, so manually post a raw completion message
    // that the executor understands.
    __wptrunner_testdriver_callback([__wptrunner_url, "complete", []]);
  }
}

// The `initialized` flag ensures only up to one `load` handler or
// `MutationObserver` is ever registered.
if (!initialized) {
  if (document.readyState != "complete") {
    addEventListener('load', wait_load, { once: true });
  } else {
    wait_load();
  }
}
if (window.__wptrunner_process_next_event) {
  window.__wptrunner_process_next_event();
}
