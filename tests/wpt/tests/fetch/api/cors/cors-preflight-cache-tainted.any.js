// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=../resources/utils.js

const host_info = get_host_info();
const redir_url = host_info.HTTP_REMOTE_ORIGIN + dirname(location.pathname) +
    RESOURCES_DIR + 'redirect.py';
const cors_url = host_info.HTTP_ORIGIN_WITH_DIFFERENT_PORT +
    dirname(location.pathname) + RESOURCES_DIR + 'preflight.py';

promise_test((test) => {
  const uuid_token = token();
  const target_url = cors_url + '?token=' + uuid_token +
      '&max_age=12000&allow_methods=POST' +
      '&allow_headers=x-test-header';
  const redirect_url = redir_url +
      '?allow_headers=x-test-header&location=' + encodeURIComponent(target_url);

  // Cross-origin redirect to target: preflight is sent with tainted origin
  // (Origin: null).
  return fetch(cors_url + '?token=' + uuid_token + '&clear-stash')
      .then(() => {
        return fetch(new Request(redirect_url, {
          mode: 'cors',
          method: 'POST',
          headers: [['x-test-header', 'test1']]
        }));
      })
      .then((resp) => {
        assert_equals(resp.status, 200, 'Response status is 200');
        assert_equals(resp.headers.get('x-did-preflight'), '1',
                      'Tainted request performed preflight');
        return fetch(cors_url + '?token=' + uuid_token + '&clear-stash');
      })
      .then((res) => res.text())
      .then((txt) => {
        assert_equals(txt, '1',
                      'Preflight stash was recorded for tainted request');
        // Direct (untainted) request to target: must NOT reuse tainted
        // preflight cache.
        return fetch(new Request(target_url, {
          mode: 'cors',
          method: 'POST',
          headers: [['x-test-header', 'test2']]
        }));
      })
      .then((resp) => {
        assert_equals(resp.status, 200, 'Response status is 200');
        assert_equals(resp.headers.get('x-did-preflight'), '1',
                      'Untainted request performed new preflight');
        return fetch(cors_url + '?token=' + uuid_token + '&clear-stash');
      })
      .then((res) => res.text())
      .then((txt) => {
        assert_equals(txt, '1',
                      'Preflight stash was recorded for untainted request');
      });
}, 'Tainted CORS preflight cache entry is not reused for untainted requests');

promise_test((test) => {
  const uuid_token = token();
  const target_url = cors_url + '?token=' + uuid_token +
      '&max_age=12000&allow_methods=POST' +
      '&allow_headers=x-test-header';
  const redirect_url = redir_url +
      '?allow_headers=x-test-header&location=' + encodeURIComponent(target_url);

  // Direct (untainted) request to target: preflight is sent with untainted
  // origin.
  return fetch(cors_url + '?token=' + uuid_token + '&clear-stash')
      .then(() => {
        return fetch(new Request(target_url, {
          mode: 'cors',
          method: 'POST',
          headers: [['x-test-header', 'test1']]
        }));
      })
      .then((resp) => {
        assert_equals(resp.status, 200, 'Response status is 200');
        assert_equals(resp.headers.get('x-did-preflight'), '1',
                      'Untainted request performed preflight');
        return fetch(cors_url + '?token=' + uuid_token + '&clear-stash');
      })
      .then((res) => res.text())
      .then((txt) => {
        assert_equals(txt, '1',
                      'Preflight stash was recorded for untainted request');
        // Cross-origin redirected (tainted) request to target: must NOT
        // reuse untainted preflight cache.
        return fetch(new Request(redirect_url, {
          mode: 'cors',
          method: 'POST',
          headers: [['x-test-header', 'test2']]
        }));
      })
      .then((resp) => {
        assert_equals(resp.status, 200, 'Response status is 200');
        assert_equals(resp.headers.get('x-did-preflight'), '1',
                      'Tainted request performed new preflight');
        return fetch(cors_url + '?token=' + uuid_token + '&clear-stash');
      })
      .then((res) => res.text())
      .then((txt) => {
        assert_equals(txt, '1',
                      'Preflight stash was recorded for tainted request');
      });
}, 'Untainted CORS preflight cache entry is not reused for tainted requests');
