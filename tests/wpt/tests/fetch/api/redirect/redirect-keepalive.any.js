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

/**
 * In an iframe, and in `load` event handler, test to fetch a keepalive URL that
 * involves in redirect to another URL.
 *
 * `unloadIframe` to unload the iframe before verifying stashed token to
 * simulate the situation that unloads after fetching. Note that this test is
 * different from `keepaliveRedirectInUnloadTest()` in that the the latter
 * performs fetch() call directly in `unload` event handler, while this test
 * does it in `load`.
 */
function keepaliveRedirectTest(
    desc,
    {origin1 = '',
     origin2 = '',
     withPreflight = false,
     unloadIframe = false} = {}) {
  desc = `[keepalive][iframe][load] ${desc}` +
      (unloadIframe ? ' [unload at end]' : '');
  promise_test(async (test) => {
    const tokenToStash = token();
    const iframe = document.createElement('iframe');
    iframe.src = getKeepAliveAndRedirectIframeUrl(
        tokenToStash, origin1, origin2, withPreflight);
    document.body.appendChild(iframe);
    await iframeLoaded(iframe);
    assert_equals(await getTokenFromMessage(), tokenToStash);
    if (unloadIframe) {
      iframe.remove();
    }

    assertStashedTokenAsync(desc, tokenToStash);
  }, `${desc}; setting up`);
}

/**
 * Opens a different site window, and in `unload` event handler, test to fetch
 * a keepalive URL that involves in redirect to another URL.
 */
function keepaliveRedirectInUnloadTest(desc, {
  origin1 = '',
  origin2 = '',
  url2 = '',
  withPreflight = false,
  expectFetchSucceed = true
} = {}) {
  desc = `[keepalive][new window][unload] ${desc}`;

  promise_test(async (test) => {
    const targetUrl =
        `${HTTP_NOTSAMESITE_ORIGIN}/fetch/api/resources/keepalive-redirect-window.html?` +
        `origin1=${origin1}&` +
        `origin2=${origin2}&` +
        `url2=${url2}&` + (withPreflight ? `with-headers` : ``);
    const w = window.open(targetUrl);
    const token = await getTokenFromMessage();
    w.close();

    assertStashedTokenAsync(
        desc, token, {expectTokenExist: expectFetchSucceed});
  }, `${desc}; setting up`);
}

for (const unloadIframe of [false, true]) {
  keepaliveRedirectTest(`same-origin redirect`, {unloadIframe});
  keepaliveRedirectTest(
      `same-origin redirect + preflight`, {withPreflight: true, unloadIframe});
  keepaliveRedirectTest(`cross-origin redirect`, {
    origin1: HTTP_REMOTE_ORIGIN,
    origin2: HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT,
    unloadIframe
  });
  keepaliveRedirectTest(`cross-origin redirect + preflight`, {
    origin1: HTTP_REMOTE_ORIGIN,
    origin2: HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT,
    withPreflight: true,
    unloadIframe
  });
}

keepaliveRedirectInUnloadTest('same-origin redirect');
keepaliveRedirectInUnloadTest(
    'same-origin redirect + preflight', {withPreflight: true});
keepaliveRedirectInUnloadTest('cross-origin redirect', {
  origin1: HTTP_REMOTE_ORIGIN,
  origin2: HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT
});
keepaliveRedirectInUnloadTest('cross-origin redirect + preflight', {
  origin1: HTTP_REMOTE_ORIGIN,
  origin2: HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT,
  withPreflight: true
});
keepaliveRedirectInUnloadTest(
    'redirect to file URL',
    {url2: 'file://tmp/bar.txt', expectFetchSucceed: false});
keepaliveRedirectInUnloadTest('redirect to data URL', {
  url2: 'data:text/plain;base64,cmVzcG9uc2UncyBib2R5',
  expectFetchSucceed: false
});
