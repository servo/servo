// META: script=/common/get-host-info.sub.js

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function fetch_redirect_test(origin, target_origin, expectation) {
  const target_url = target_origin + '/common/blank-with-cors.html';
  const url = origin + '/common/redirect.py?status=302&location=' +
      encodeURIComponent(target_url);

  if (expectation === FAILURE) {
    return promise_test(async t => {
      const fetcher = fetch(url, {mode: 'cors', credentials: 'omit'});
      return promise_rejects_js(t, TypeError, fetcher);
    }, `Fetch redirect from ${origin} to ${target_origin} fails.`);
  }

  promise_test(async t => {
    const r = await fetch(url, {mode: 'cors', credentials: 'omit'});
    assert_equals(r.status, 200);
  }, `Fetch redirect from ${origin} to ${target_origin} succeeds.`);
}
