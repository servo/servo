const url_base = "/permissions-policy/experimental-features/resources/";
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

// Returns a promise which is resolved when |load| event is dispatched for |e|.
function wait_for_load(e) {
  return new Promise((resolve) => {
    e.addEventListener("load", resolve);
  });
}

setup(() => {
  window.reporting_observer_instance = new ReportingObserver((reports, observer) => {
    if (window.reporting_observer_callback) {
      reports.forEach(window.reporting_observer_callback);
    }
  }, {types: ["permissions-policy-violation"]});
  window.reporting_observer_instance.observe();
  window.reporting_observer_callback = null;
});

// Waits for a violation in |feature| and source file containing |file_name|.
function wait_for_violation_in_file(feature, file_name) {
  return new Promise( (resolve) => {
    assert_equals(null, window.reporting_observer_callback);
    window.reporting_observer_callback = (report) => {
        var feature_match = (feature === report.body.featureId);
        var file_name_match =
            !file_name ||
            (report.body.sourceFile.indexOf(file_name) !== -1);
        if (feature_match && file_name_match) {
          window.reporting_observer_callback = null;
          resolve(report);
        }
    };
  });
}
