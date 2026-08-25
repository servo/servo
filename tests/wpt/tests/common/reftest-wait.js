/**
 * Remove the `reftest-wait` class on the document element.
 * The reftest runner will wait with taking a screenshot while
 * this class is present.
 *
 * See https://web-platform-tests.org/writing-tests/reftests.html#controlling-when-comparison-occurs
 */
function takeScreenshot() {
    document.documentElement.classList.remove("reftest-wait");
}

/**
 * Call `takeScreenshot()` after a delay of at least |timeout| milliseconds.
 * @param {number} timeout - milliseconds
 */
function takeScreenshotDelayed(timeout) {
    setTimeout(function() {
        takeScreenshot();
    }, timeout);
}

/**
 * Ensure that a precondition is met before waiting for a screenshot.
 * @param {bool} condition - Fail the test if this evaluates to false
 * @param {string} msg - Error message to write to the screenshot
 * @returns {bool} True if the condition passed, false if it failed
 */
function failIfNot(condition, msg) {
  const fail = () => {
    (document.body || document.documentElement).textContent = `Precondition Failed: ${msg}`;
    takeScreenshot();
  };
  if (!condition) {
    if (document.readyState == "interactive") {
      fail();
    } else {
      document.addEventListener("DOMContentLoaded", fail, false);
    }
    return false;
  }
  return true;
}

/**
 * Display the failure reason and stop waiting for a screenshot.
 *
 * Does nothing for a mismatch-only reftest (no `match` reference is also
 * present), since an error page necessarily differs from the reference and
 * would spuriously pass instead of correctly timing out.
 * @param {Error|ErrorEvent|PromiseRejectionEvent|*} error - The error that caused the failure.
 */
function failOnError(error) {
  // Only <link>s explicitly in the XHTML namespace count as match/mismatch
  // links, matching the manifest's own namespace-qualified lookup.
  const links = document.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "link");
  let hasMatch = false;
  let hasMismatch = false;
  for (const link of links) {
    const rel = link.getAttribute("rel");
    if (rel === "match") {
      hasMatch = true;
    } else if (rel === "mismatch") {
      hasMismatch = true;
    }
  }
  if (hasMismatch && !hasMatch) {
    return;
  }

  let message;
  if (typeof PromiseRejectionEvent !== "undefined" && error instanceof PromiseRejectionEvent) {
    if (error.reason && error.reason.message) {
      message = "Unhandled rejection: " + error.reason.message;
    } else {
      message = "Unhandled rejection";
    }
  } else {
    if (error.message) {
      message = "Uncaught exception: " + error.message;
    } else {
      message = "Uncaught exception";
    }
  }

  const node = document.createElementNS("http://www.w3.org/1999/xhtml", "div");
  node.textContent = message;

  if (document.body) {
    document.body.insertBefore(node, document.body.firstChild);
  } else {
    const root = document.documentElement;
    const is_html = (root &&
                     root.namespaceURI === "http://www.w3.org/1999/xhtml" &&
                     root.localName === "html");
    const is_svg = ("SVGSVGElement" in self && root instanceof SVGSVGElement);
    if (is_svg) {
      const foreignObject = document.createElementNS("http://www.w3.org/2000/svg", "foreignObject");
      foreignObject.setAttribute("width", "100%");
      foreignObject.setAttribute("height", "100%");
      root.insertBefore(foreignObject, root.firstChild);
      foreignObject.appendChild(node);
    } else if (is_html) {
      root.appendChild(document.createElementNS("http://www.w3.org/1999/xhtml", "body"))
          .appendChild(node);
    } else {
      root.insertBefore(node, root.firstChild);
    }
  }

  takeScreenshot();
}

/**
 * Wrap `func` so a synchronous exception it throws is reported via
 * failOnError() instead of propagating as an uncaught error. For use
 * wrapping a callback, e.g. an event listener.
 * @param {Function} func - Callback to wrap.
 * @returns {Function} The wrapped callback.
 */
function reftestStep(func) {
  return function(...args) {
    try {
      return func.apply(this, args);
    } catch (e) {
      failOnError(e);
    }
  };
}

/**
 * Call `func` immediately, reporting via failOnError() either a
 * synchronous exception it throws or an asynchronous rejection of the
 * promise it returns.
 * @param {Function} func - Function to call, typically async.
 */
function reftestPromise(func) {
  const result = reftestStep(func)();
  Promise.resolve(result).catch(failOnError);
}

/**
 * Once a text track cue becomes active, pause the video, wait
 * for layout to update, then call takeScreenshot().
 */
function waitForActiveCueAndTakeScreenshot() {
    var videoElement = document.querySelector("video");
    var trackElement = document.querySelector("track");

    if (!failIfNot(videoElement, "Video element not found"))
        return;

    if (!failIfNot(trackElement, "Track element not found"))
        return;

    var textTrack = trackElement.track;

    function pauseVideoAndTakeScreenshot() {
        if (videoElement.paused)
            requestAnimationFrame(() => takeScreenshot());
        else {
            videoElement.addEventListener("pause", function() {
                requestAnimationFrame(() => takeScreenshot());
            });
            videoElement.pause();
        }
    }

    textTrack.oncuechange = function() {
        if (textTrack.activeCues && textTrack.activeCues.length) {
            textTrack.oncuechange = null;
            pauseVideoAndTakeScreenshot();
        }
    };

    if (textTrack.activeCues && textTrack.activeCues.length)
        pauseVideoAndTakeScreenshot();
}

