function assert_no_csp_event_for_url(test, url) {
  self.addEventListener("securitypolicyviolation", test.step_func(e => {
    if (e.blockedURI !== url)
      return;
    assert_unreached("SecurityPolicyViolation event fired for " + url);
  }));
}

function assert_no_event(test, obj, name) {
  obj.addEventListener(name, test.unreached_func("The '" + name + "' event should not have fired."));
}

function waitUntilCSPEventForURL(test, url) {
  return new Promise((resolve, reject) => {
    self.addEventListener("securitypolicyviolation", test.step_func(e => {
      if (e.blockedURI == url)
        resolve(e);
    }));
  });
}

function waitUntilCSPEventForEval(test, line) {
  return new Promise((resolve, reject) => {
    self.addEventListener("securitypolicyviolation", test.step_func(e => {
      if (e.blockedURI == "eval" && e.lineNumber == line)
        resolve(e);
    }));
  });
}

function waitUntilEvent(obj, name) {
  return new Promise((resolve, reject) => {
    obj.addEventListener(name, resolve);
  });
}

// Given the URL of a worker that pings its opener upon load, this
// function builds a test that asserts that the ping is received,
// and that no CSP event fires.
function assert_worker_is_loaded(url, description) {
  async_test(t => {
    assert_no_csp_event_for_url(t, url);
    var w = new Worker(url);
    assert_no_event(t, w, "error");
    waitUntilEvent(w, "message")
      .then(t.step_func_done(e => {
        assert_equals(e.data, "ping");
      }));
  }, description);
}

function assert_shared_worker_is_loaded(url, description) {
  async_test(t => {
    assert_no_csp_event_for_url(t, url);
    var w = new SharedWorker(url);
    assert_no_event(t, w, "error");
    waitUntilEvent(w.port, "message")
      .then(t.step_func_done(e => {
        assert_equals(e.data, "ping");
      }));
    w.port.start();
  }, description);
}

function assert_service_worker_is_loaded(url, description) {
  promise_test(t => {
    assert_no_csp_event_for_url(t, url);
    return Promise.all([
      waitUntilEvent(navigator.serviceWorker, "message")
        .then(e => {
          assert_equals(e.data, "ping");
        }),
      navigator.serviceWorker.register(url, { scope: url })
        .then(r => {
          var sw = r.active || r.installing || r.waiting;
          t.add_cleanup(_ => r.unregister());
          sw.postMessage("pong?");
        })
    ]);
  }, description);
}

// Given the URL of a worker that pings its opener upon load, this
// function builds a test that asserts that the constructor throws
// a SecurityError, and that a CSP event fires.
function assert_worker_is_blocked(url, description) {
  async_test(t => {
    // If |url| is a blob, it will be stripped down to "blob" for reporting.
    var reportedURL = new URL(url).protocol == "blob:" ? "blob" : url;
    waitUntilCSPEventForURL(t, reportedURL)
      .then(t.step_func_done(e => {
        assert_equals(e.blockedURI, reportedURL);
        assert_equals(e.violatedDirective, "worker-src");
        assert_equals(e.effectiveDirective, "worker-src");
      }));

    // TODO(mkwst): We shouldn't be throwing here. We should be firing an
    // `error` event on the Worker. https://crbug.com/663298
    assert_throws("SecurityError", function () {
      var w = new Worker(url);
    });
  }, description);
}

function assert_shared_worker_is_blocked(url, description) {
  async_test(t => {
    // If |url| is a blob, it will be stripped down to "blob" for reporting.
    var reportedURL = new URL(url).protocol == "blob:" ? "blob" : url;
    waitUntilCSPEventForURL(t, reportedURL)
      .then(t.step_func_done(e => {
        assert_equals(e.blockedURI, reportedURL);
        assert_equals(e.violatedDirective, "worker-src");
        assert_equals(e.effectiveDirective, "worker-src");
      }));

    // TODO(mkwst): We shouldn't be throwing here. We should be firing an
    // `error` event on the SharedWorker. https://crbug.com/663298
    assert_throws("SecurityError", function () {
      var w = new SharedWorker(url);
    });
  }, description);
}

function assert_service_worker_is_blocked(url, description) {
  promise_test(t => {
    assert_no_event(t, navigator.serviceWorker, "message");
    // If |url| is a blob, it will be stripped down to "blob" for reporting.
    var reportedURL = new URL(url).protocol == "blob:" ? "blob" : url;
    return Promise.all([
      waitUntilCSPEventForURL(t, reportedURL)
        .then(t.step_func_done(e => {
          assert_equals(e.blockedURI, reportedURL);
          assert_equals(e.violatedDirective, "worker-src");
          assert_equals(e.effectiveDirective, "worker-src");
        })),
      promise_rejects(t, "SecurityError", navigator.serviceWorker.register(url, { scope: url }))
    ]);
  }, description);
}
