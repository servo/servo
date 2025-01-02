importScripts("{{location[server]}}/resources/testharness.js");
importScripts("{{location[server]}}/content-security-policy/support/testharness-helper.js");

let base_same_origin_url =
      "{{location[server]}}/content-security-policy/support/resource.py";

// Same-origin
promise_test(t => {
  let url = `${base_same_origin_url}?same-origin-fetch`;
  assert_no_csp_event_for_url(t, url);

  return fetch(url)
      .then(t.step_func(r => assert_equals(r.status, 200)));
}, "Same-origin 'fetch()'.");

// XHR is not available in service workers.
if (self.XMLHttpRequest) {
  promise_test(t => {
    let url = `${base_same_origin_url}?same-origin-xhr`;
    assert_no_csp_event_for_url(t, url);

    return new Promise((resolve, reject) => {
      var xhr = new XMLHttpRequest();
      xhr.open("GET", url);
      xhr.onload = resolve;
      xhr.onerror = _ => reject("xhr.open should success.");
      xhr.send();
    });
  }, "Same-origin XHR.");
}

let base_cross_origin_url =
      "https://{{hosts[][www]}}:{{ports[https][1]}}" +
      "/content-security-policy/support/resource.py";
let fetch_cross_origin_url = `${base_cross_origin_url}?cross-origin-fetch`;

// Cross-origin
promise_test(t => {
  let url = fetch_cross_origin_url;

  return Promise.all([
    waitUntilCSPEventForURL(t, url),
    fetch(url)
  ]);
}, "Cross-origin 'fetch()'.");

let xhr_cross_origin_url = `${base_cross_origin_url}?cross-origin-xhr`;

// XHR is not available in service workers.
if (self.XMLHttpRequest) {
  promise_test(t => {
    let url = xhr_cross_origin_url;

    return Promise.all([
      waitUntilCSPEventForURL(t, url),
      new Promise((resolve, reject) => {
        var xhr = new XMLHttpRequest();
        xhr.open("GET", url);
        xhr.onload = resolve;
        xhr.onerror = _ => reject("xhr.open should not have thrown.");
        xhr.send();
      })
    ]);
  }, "Cross-origin XHR.");
}

let redirect_url = `{{location[server]}}/common/redirect-opt-in.py?` +
      `status=307&location=${fetch_cross_origin_url}`;

// Same-origin redirecting to cross-origin
promise_test(t => {
  let url = redirect_url;

  return Promise.all([
    waitUntilCSPEventForURL(t, url),
    fetch(url)
  ]);
}, "Same-origin => cross-origin 'fetch()'.");

let websocket_url = "wss://{{host}}:{{ports[wss][0]}}/echo";

// The WebSocket URL is not the same as 'self'
promise_test(t => {
  return Promise.all([
    waitUntilCSPEventForURL(t, websocket_url),
    new Promise(resolve => {
      let ws = new WebSocket(websocket_url);
      ws.onopen = resolve;
    })
  ]);
}, "WebSocket.");

let expected_blocked_urls = self.XMLHttpRequest
    ? [ fetch_cross_origin_url, xhr_cross_origin_url, redirect_url, websocket_url ]
    : [ fetch_cross_origin_url, redirect_url, websocket_url ];

promise_test(async t => {
  let report_url = `{{location[server]}}/reporting/resources/report.py?` +
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
        x["disposition"], "report",
        "Disposition in report does not match");
    assert_equals(
        x["document-uri"],
        "{{location[server]}}/content-security-policy/inside-worker/" +
          "support/connect-src-self-report-only.sub.js?id={{GET[id]}}",
        "Document uri in report does not match");
  });
});

done();
