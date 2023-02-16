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
}, "Same-origin 'fetch()' in " + self.location.protocol + " without CSP");

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
  }, "Same-origin XHR in " + self.location.protocol + " without CSP");
}

// Cross-origin
promise_test(t => {
  let url = `${base_cross_origin_url}?cross-origin-fetch`;
  assert_no_csp_event_for_url(t, url);

  return fetch(url)
    .then(t.step_func(r => assert_equals(r.status, 200)));
}, "Cross-origin 'fetch()' in " + self.location.protocol + " without CSP");

// XHR is not available in service workers.
if (self.XMLHttpRequest) {
  promise_test(t => {
    let url = `${base_cross_origin_url}?cross-origin-xhr`;
    assert_no_csp_event_for_url(t, url);

    return new Promise((resolve, reject) => {
      let xhr = new XMLHttpRequest();
      xhr.open("GET", url);
      xhr.onload = resolve;
      xhr.onerror = _ => reject("xhr.open should success.");
      xhr.send();
    });
  }, "Cross-origin XHR in " + self.location.protocol + " without CSP");
}

// Same-origin redirecting to cross-origin
promise_test(t => {
  let url = `{{location[server]}}/common/redirect-opt-in.py?` +
      `status=307&location=${base_cross_origin_url}?cross-origin-fetch`;
  assert_no_csp_event_for_url(t, url);

  return fetch(url)
    .then(t.step_func(r => assert_equals(r.status, 200)));
}, "Same-origin => cross-origin 'fetch()' in " + self.location.protocol +
           " without CSP");

// WebSocket
promise_test(async function(t) {
  let url = "wss://{{host}}:{{ports[wss][0]}}/echo";
  assert_no_csp_event_for_url(t, url);

  return new Promise(resolve => {
    let ws = new WebSocket(url);
    ws.onopen = resolve;
  });
}, "WebSocket without CSP");

done();
