// META: global=window,worker
// META: script=../resources/utils.js

// Tests receiving a redirect response with a Location header with an empty
// value.

const url = RESOURCES_DIR + 'redirect-empty-location.py';

promise_test(t => {
  return fetch(url, {redirect:'manual'})
    .then(resp => {
      assert_equals(resp.type, 'opaqueredirect');
      assert_equals(resp.status, 0);
    });
}, 'redirect response with empty Location, manual mode');

// In "error" mode any redirect status is a network error, whatever the
// Location header holds.
for (const status of [301, 302, 303, 307, 308]) {
  for (const redirect of ['follow', 'error']) {
    promise_test(t => {
      return promise_rejects_js(
          t, TypeError, fetch(`${url}?status=${status}`, {redirect}));
    }, `${status} redirect response with empty Location, ${redirect} mode`);
  }
}

done();
