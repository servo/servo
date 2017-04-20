importScripts("{{location[server]}}/resources/testharness.js");
importScripts("{{location[server]}}/content-security-policy/support/testharness-helper.js");

// Same-origin
async_test(t => {
  var url = "{{location[server]}}/content-security-policy/support/resource.py?same-origin-fetch";
  assert_no_csp_event_for_url(t, url);

  fetch(url)
    .then(t.step_func_done(r => assert_equals(r.status, 200)));
}, "Same-origin 'fetch()' in " + self.location.protocol + self.location.search);

async_test(t => {
  var url = "{{location[server]}}/content-security-policy/support/resource.py?same-origin-xhr";
  assert_no_csp_event_for_url(t, url);

  var xhr = new XMLHttpRequest();
  xhr.open("GET", url);
  xhr.onload = t.step_func_done();
  xhr.onerror = t.unreached_func();
  xhr.send();
}, "Same-origin XHR in " + self.location.protocol + self.location.search);

// Cross-origin
async_test(t => {
  var url = "http://{{domains[www]}}:{{ports[http][1]}}/content-security-policy/support/resource.py?cross-origin-fetch";
  assert_no_csp_event_for_url(t, url);

  fetch(url)
    .then(t.step_func_done(r => assert_equals(r.status, 200)));
}, "Cross-origin 'fetch()' in " + self.location.protocol + self.location.search);

async_test(t => {
  var url = "http://{{domains[www]}}:{{ports[http][1]}}/content-security-policy/support/resource.py?cross-origin-xhr";
  assert_no_csp_event_for_url(t, url);

  var xhr = new XMLHttpRequest();
  xhr.open("GET", url);
  xhr.onload = t.step_func_done();
  xhr.onerror = t.unreached_func();
  xhr.send();
}, "Cross-origin XHR in " + self.location.protocol + self.location.search);

// Same-origin redirecting to cross-origin
async_test(t => {
  var url = "{{location[server]}}/common/redirect-opt-in.py?status=307&location=http://{{domains[www]}}:{{ports[http][1]}}/content-security-policy/support/resource.py?cross-origin-fetch";
  assert_no_csp_event_for_url(t, url);

  fetch(url)
    .then(t.step_func_done(r => assert_equals(r.status, 200)));
}, "Same-origin => cross-origin 'fetch()' in " + self.location.protocol + self.location.search);

done();
