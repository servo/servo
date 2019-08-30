importScripts("{{location[server]}}/resources/testharness.js");
importScripts("{{location[server]}}/content-security-policy/support/testharness-helper.js");

// Same-origin
promise_test(t => {
  var url = "{{location[server]}}/common/text-plain.txt?same-origin-fetch";
  assert_no_csp_event_for_url(t, url);

  return fetch(url)
    .then(t.step_func(r => assert_equals(r.status, 200)));
}, "Same-origin 'fetch()' in " + self.location.protocol + self.location.search);

promise_test(t => {
  var url = "{{location[server]}}/common/text-plain.txt?same-origin-xhr";
  assert_no_csp_event_for_url(t, url);

  return new Promise((resolve, reject) => {
    var xhr = new XMLHttpRequest();
    xhr.open("GET", url);
    xhr.onload = t.step_func(resolve);
    xhr.onerror = t.step_func(_ => reject("xhr.open should success."));
    xhr.send();
  });
}, "Same-origin XHR in " + self.location.protocol + self.location.search);

// Cross-origin
promise_test(t => {
  var url = "http://{{domains[www]}}:{{ports[http][1]}}/common/text-plain.txt?cross-origin-fetch";

  return Promise.all([
    // TODO(mkwst): A 'securitypolicyviolation' event should fire.
    fetch(url)
      .catch(t.step_func(e => assert_true(e instanceof TypeError)))
  ]);
}, "Cross-origin 'fetch()' in " + self.location.protocol + self.location.search);

promise_test(t => {
  var url = "http://{{domains[www]}}:{{ports[http][1]}}/common/text-plain.txt?cross-origin-xhr";

  return Promise.all([
    // TODO(mkwst): A 'securitypolicyviolation' event should fire.
    new Promise((resolve, reject) => {
      var xhr = new XMLHttpRequest();
      xhr.open("GET", url);
      xhr.onload = t.step_func(_ => reject("xhr.open should have thrown."));
      xhr.onerror = t.step_func(resolve);
      xhr.send();
    })
  ]);
}, "Cross-origin XHR in " + self.location.protocol + self.location.search);

// Same-origin redirecting to cross-origin
promise_test(t => {
  var url = "{{location[server]}}/common/redirect-opt-in.py?status=307&location=http://{{domains[www]}}:{{ports[http][1]}}/common/text-plain.txt?cross-origin-fetch";

  // TODO(mkwst): A 'securitypolicyviolation' event should fire.
  return promise_rejects(t, new TypeError, fetch(url));
}, "Same-origin => cross-origin 'fetch()' in " + self.location.protocol + self.location.search);

done();
