importScripts("{{location[server]}}/resources/testharness.js");
importScripts("{{location[server]}}/content-security-policy/support/testharness-helper.js");

let base_same_origin_url =
      "{{location[server]}}/content-security-policy/support/resource.py";
let base_cross_origin_url =
      "https://{{hosts[][www]}}:{{ports[https][1]}}" +
      "/content-security-policy/support/resource.py";

// Same-origin
promise_test(t => {
  let url = `${base_same_origin_url}?same-origin-fetch`;
  assert_no_csp_event_for_url(t, url);

  return fetch(url)
    .then(t.step_func(r => assert_equals(r.status, 200)));
}, "Same-origin 'fetch()' in " + self.location.protocol +
             " with {{GET[test-name]}}");

// XHR is not available in service workers.
if (self.XMLHttpRequest) {
  promise_test(t => {
    let url = `${base_same_origin_url}?same-origin-xhr`;
    assert_no_csp_event_for_url(t, url);

    return new Promise((resolve, reject) => {
      let xhr = new XMLHttpRequest();
      xhr.open("GET", url);
      xhr.onload = resolve;
      xhr.onerror = _ => reject("xhr.open should success.");
      xhr.send();
    });
  }, "Same-origin XHR in " + self.location.protocol +
               " with {{GET[test-name]}}");
}

let fetch_cross_origin_url = `${base_cross_origin_url}?cross-origin-fetch`;

// Cross-origin
promise_test(t => {
  let url = fetch_cross_origin_url;

  return Promise.all([
    waitUntilCSPEventForURL(t, url),
    fetch(url)
        .then(t.step_func(_ => assert_unreached(
            "cross-origin fetch should have thrown.")))
        .catch(t.step_func(e => assert_true(e instanceof TypeError)))
  ]);
}, "Cross-origin 'fetch()' in " + self.location.protocol +
             " with {{GET[test-name]}}");

let xhr_cross_origin_url = `${base_cross_origin_url}?cross-origin-xhr`;

// XHR is not available in service workers.
if (self.XMLHttpRequest) {
  promise_test(t => {
    let url = xhr_cross_origin_url;

    return Promise.all([
      waitUntilCSPEventForURL(t, url),
      new Promise((resolve, reject) => {
        let xhr = new XMLHttpRequest();
        xhr.open("GET", url);
        xhr.onload = _ => reject("xhr.open should have thrown.");
        xhr.onerror = resolve;
        xhr.send();
      })
    ]);
  }, "Cross-origin XHR in " + self.location.protocol +
               " with {{GET[test-name]}}");
}

let redirect_url = `{{location[server]}}/common/redirect-opt-in.py?` +
      `status=307&location=${fetch_cross_origin_url}`;

// Same-origin redirecting to cross-origin
promise_test(t => {
  let url = redirect_url;

  return Promise.all([
    waitUntilCSPEventForURL(t, url),
    fetch(url)
        .then(t.step_func(_ => assert_unreached(
            "cross-origin redirect should have thrown.")))
      .catch(t.step_func(e => assert_true(e instanceof TypeError)))
  ]);
}, "Same-origin => cross-origin 'fetch()' in " + self.location.protocol +
             " with {{GET[test-name]}}");


let websocket_url = "wss://{{host}}:{{ports[wss][0]}}/echo";

// The WebSocket URL is not the same as 'self'
promise_test(t => {
  return Promise.all([
    waitUntilCSPEventForURL(t, websocket_url),
    new Promise((resolve, reject) => {
      // Firefox throws in the constructor, Chrome triggers the error event.
      try {
        let ws = new WebSocket(websocket_url);
        ws.onerror = resolve;
        ws.onopen = reject; // unexpected
      } catch (e) {
        resolve();
      }
    })
  ]);
}, "WebSocket in " + self.location.protocol + " with {{GET[test-name]}}");

let expected_blocked_urls = self.XMLHttpRequest
    ? [ fetch_cross_origin_url, xhr_cross_origin_url, redirect_url, websocket_url ]
    : [ fetch_cross_origin_url, redirect_url, websocket_url ];

promise_test(async t => {
  let report_url = `{{location[server]}}/reporting/resources/report.py` +
      `?op=retrieve_report&reportID={{GET[id]}}` +
      `&min_count=${expected_blocked_urls.length}`;

  let response = await fetch(report_url);
  assert_equals(response.status, 200, "Fetching reports failed");

  let response_json = await response.json();
  let reports = response_json.map(x => x["csp-report"]);

  assert_array_equals(
      reports.map(x => x["blocked-uri"]).sort(),
      expected_blocked_urls.sort(),
      "Reports do not match");
  reports.forEach(x => {
    assert_equals(
        x["violated-directive"], "connect-src",
        "Violated directive in report does not match");
    assert_equals(
        x["effective-directive"], "connect-src",
        "Effective directive in report does not match");
    assert_equals(
        x["disposition"], "enforce",
        "Effective directive in report does not match");
  });
}, "Reports match in " + self.location.protocol + " with {{GET[test-name]}}");

done();
