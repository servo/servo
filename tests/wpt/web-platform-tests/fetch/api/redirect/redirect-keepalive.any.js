// META: global=window
// META: title=Fetch API: keepalive handling
// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=../resources/keepalive-helper.js

'use strict';

const {
  HTTP_NOTSAMESITE_ORIGIN,
  HTTP_REMOTE_ORIGIN,
  HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT
} = get_host_info();

promise_test(async (test) => {
  const token1 = token();
  const iframe = document.createElement('iframe');
  iframe.src = getKeepAliveAndRedirectIframeUrl(
      token1, '', '', /*withPreflight=*/ false);
  document.body.appendChild(iframe);
  await iframeLoaded(iframe);
  assert_equals(await getTokenFromMessage(), token1);
  iframe.remove();

  assertStashedTokenAsync('same-origin redirect', token1);
}, 'same-origin redirect; setting up');

promise_test(async (test) => {
  const token1 = token();
  const iframe = document.createElement('iframe');
  iframe.src = getKeepAliveAndRedirectIframeUrl(
      token1, HTTP_REMOTE_ORIGIN, HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT,
      /*withPreflight=*/ false);
  document.body.appendChild(iframe);
  await iframeLoaded(iframe);
  assert_equals(await getTokenFromMessage(), token1);
  iframe.remove();

  assertStashedTokenAsync('cross-origin redirect', token1);
}, 'cross-origin redirect; setting up');

promise_test(async (test) => {
  const token1 = token();
  const iframe = document.createElement('iframe');
  iframe.src = getKeepAliveAndRedirectIframeUrl(
      token1, HTTP_REMOTE_ORIGIN, HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT,
      /*withPreflight=*/ true);
  document.body.appendChild(iframe);
  await iframeLoaded(iframe);
  assert_equals(await getTokenFromMessage(), token1);
  iframe.remove();

  assertStashedTokenAsync('cross-origin redirect with preflight', token1);
}, 'cross-origin redirect with preflight; setting up');
