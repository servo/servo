// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=beacon-common.sub.js

'use strict';

const {HTTPS_ORIGIN, ORIGIN, HTTPS_REMOTE_ORIGIN} = get_host_info();

// As /common/redirect.py is not under our control, let's make sure that
// it doesn't support CORS.
parallelPromiseTest(async (t) => {
  const destination = `${HTTPS_REMOTE_ORIGIN}/common/text-plain.txt` +
      `?pipe=header(access-control-allow-origin,*)`;
  const redirect = `${HTTPS_REMOTE_ORIGIN}/common/redirect.py` +
      `?location=${encodeURIComponent(destination)}`;

  // Fetching `destination` is fine because it supports CORS.
  await fetch(destination);

  // Fetching redirect.py should fail because it doesn't support CORS.
  await promise_rejects_js(t, TypeError, fetch(redirect));
}, '/common/redirect.py does not support CORS');

for (const type of [STRING, ARRAYBUFFER, FORM, BLOB]) {
  parallelPromiseTest(async (t) => {
    const iframe = document.createElement('iframe');
    document.body.appendChild(iframe);
    t.add_cleanup(() => iframe.remove());

    const payload = makePayload(SMALL, type);
    const id = token();
    // As we use "no-cors" for CORS-safelisted requests, the redirect is
    // processed without an error while the request is cross-origin and the
    // redirect handler doesn't support CORS.
    const destination =
        `${HTTPS_REMOTE_ORIGIN}/beacon/resources/beacon.py?cmd=store&id=${id}`;
    const url = `${HTTPS_REMOTE_ORIGIN}/common/redirect.py` +
        `?status=307&location=${encodeURIComponent(destination)}`;

    assert_true(iframe.contentWindow.navigator.sendBeacon(url, payload));
    iframe.remove();

    await waitForResult(id);
  }, `cross-origin, CORS-safelisted: type = ${type}`);
}

parallelPromiseTest(async (t) => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  const payload = makePayload(SMALL, BLOB, 'application/octet-stream');
  const id = token();
  const destination =
      `${HTTPS_REMOTE_ORIGIN}/beacon/resources/beacon.py?cmd=store&id=${id}`;
  const url = `${HTTPS_REMOTE_ORIGIN}/common/redirect.py` +
      `?status=307&location=${encodeURIComponent(destination)}`;
  assert_true(iframe.contentWindow.navigator.sendBeacon(url, payload));
  iframe.remove();

  // The beacon is rejected during redirect handling because /common/redirect.py
  // doesn't support CORS.

  await new Promise((resolve) => step_timeout(resolve, 3000));
  const res = await fetch(`/beacon/resources/beacon.py?cmd=stat&id=${id}`);
  assert_equals((await res.json()).length, 0);
}, `cross-origin, non-CORS-safelisted: failure case (with redirect)`);

parallelPromiseTest(async (t) => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  const payload = makePayload(SMALL, BLOB, 'application/octet-stream');
  const id = token();
  const url =
      `${HTTPS_REMOTE_ORIGIN}/beacon/resources/beacon.py?cmd=store&id=${id}`;
  assert_true(iframe.contentWindow.navigator.sendBeacon(url, payload));
  iframe.remove();

  // The beacon is rejected during preflight handling.
  await waitForResult(id, /*expectedError=*/ 'Preflight not expected.');
}, `cross-origin, non-CORS-safelisted: failure case (without redirect)`);

for (const credentials of [false, true]) {
  parallelPromiseTest(async (t) => {
    const iframe = document.createElement('iframe');
    document.body.appendChild(iframe);
    t.add_cleanup(() => iframe.remove());

    const payload = makePayload(SMALL, BLOB, 'application/octet-stream');
    const id = token();
    let url = `${HTTPS_REMOTE_ORIGIN}/beacon/resources/beacon.py` +
        `?cmd=store&id=${id}&preflightExpected&origin=${ORIGIN}`;
    if (credentials) {
      url += `&credentials=true`;
    }
    assert_true(iframe.contentWindow.navigator.sendBeacon(url, payload));
    iframe.remove();

    // We need access-control-allow-credentials in the preflight response. This
    // shows that the request's credentials mode is 'include'.
    if (credentials) {
      const result = await waitForResult(id);
      assert_equals(result.type, 'application/octet-stream');
    } else {
      await new Promise((resolve) => step_timeout(resolve, 3000));
      const res = await fetch(`/beacon/resources/beacon.py?cmd=stat&id=${id}`);
      assert_equals((await res.json()).length, 0);
    }
  }, `cross-origin, non-CORS-safelisted[credentials=${credentials}]`);
}

parallelPromiseTest(async (t) => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  const payload = makePayload(SMALL, BLOB, 'application/octet-stream');
  const id = token();
  const destination = `${HTTPS_REMOTE_ORIGIN}/beacon/resources/beacon.py` +
      `?cmd=store&id=${id}&preflightExpected&origin=${ORIGIN}&credentials=true`;
  const url = `${HTTPS_REMOTE_ORIGIN}/fetch/api/resources/redirect.py` +
      `?redirect_status=307&allow_headers=content-type` +
      `&location=${encodeURIComponent(destination)}`;
  assert_true(iframe.contentWindow.navigator.sendBeacon(url, payload));
  iframe.remove();

  const result = await waitForResult(id);
  assert_equals(result.type, 'application/octet-stream');
}, `cross-origin, non-CORS-safelisted success-case (with redirect)`);
