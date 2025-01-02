// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
"use strict";

// These are secure origins with different relations to the current document.
const https_origin = 'https://{{host}}:{{ports[https][0]}}';
const same_site = 'https://{{hosts[][www]}}:{{ports[https][0]}}';
const cross_site = 'https://{{hosts[alt][]}}:{{ports[https][0]}}';
const alt_cross_site = 'https://{{hosts[alt][www]}}:{{ports[https][0]}}';

const responder_script = 'embedded_responder.js';
const nested_path = '/storage-access-api/resources/nested-handle-storage-access-headers.py';
const retry_path = '/storage-access-api/resources/handle-headers-retry.py';
const non_retry_path = '/storage-access-api/resources/handle-headers-non-retry.py';

function makeURL(key, domain, path, params) {
    const request_params = new URLSearchParams(params);
    request_params.append('key', key);
    return domain + path + '?' + request_params.toString();
}

async function grantStorageAccessForEmbedSite(test, origin) {
    const iframe_params = new URLSearchParams([['script', responder_script]]);
    const iframe = await CreateFrame(origin +
          '/storage-access-api/resources/script-with-cookie-header.py?' +
          iframe_params.toString());
    test.add_cleanup( async () => {
        await SetPermissionInFrame(iframe,
                                   [{ name: 'storage-access' }, 'prompt']);
        iframe.parentNode.removeChild(iframe);
    })
    await SetPermissionInFrame(iframe,
                                [{ name: 'storage-access' }, 'granted']);
}

// Sends a request whose headers can be read in cross-site contexts.
async function sendReadableHeaderRequest(url) {
    return fetch(url, {credentials: 'include', mode: 'no-cors'});
}

// Sends a request `resources/retrieve-storage-access-headers.py` and parses
// the response as JSON. Will return `undefined` if no headers were set at the
// given key, or if the headers have already been retrieved from that key.
async function sendRetrieveRequest(key) {
    const retrieval_path = '/storage-access-api/resources/retrieve-storage-access-headers.py?';
    const request_params = new URLSearchParams([['key', key]]);
    const response = await fetch(retrieval_path + request_params.toString());

    return response.status !== 200 ? undefined : response.json();
}

// Checks that the values of `actual_headers` match those passed in the
// `expected_headers` at their respective header keys. Headers with the value
// of `undefined` in `expected_headers` are expected to be absent from
// `actual_headers`.
function assertHeaderValuesMatch(actual_headers, expected_headers) {
    for (const [expected_name, expected_value] of Object.entries(
                                                          expected_headers)) {
      const actual_value = actual_headers[expected_name];
      if (expected_value === undefined) {
        assert_equals(actual_value, undefined);
      } else {
        assert_array_equals(actual_value, expected_value);
      }
    }
}

function addCommonCleanupCallback(test) {
    test.add_cleanup(async () => {
        await test_driver.delete_all_cookies();
        await MaybeSetStorageAccess("*", "*", "allowed");
      });
}

function retriedKey(key) {
    return key + 'active';
}

function redirectedKey(key) {
    return key + 'redirected';
}

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    await fetch(makeURL(key, cross_site, non_retry_path),
                {credentials: 'omit', mode: 'no-cors'});
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': undefined});
}, "Sec-Fetch-Storage-Access is omitted when credentials are omitted");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    await sendReadableHeaderRequest(makeURL(key, cross_site, non_retry_path));
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['none']});
}, "Sec-Fetch-Storage-Access is `none` when unpartitioned cookies are unavailable.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");

    // Create an iframe and grant it storage access permissions.
    await grantStorageAccessForEmbedSite(t, cross_site);
    // A cross-site request to the same site as the above iframe should have an
    // `inactive` storage access status since a permission grant exists for the
    // context.
    await sendReadableHeaderRequest(makeURL(key, cross_site, non_retry_path));
    const headers = await sendRetrieveRequest(key);
    // We should see the origin header on the inactive case.
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['inactive'],
                                      'origin': [https_origin]});
}, "Sec-Fetch-Storage-Access is `inactive` when unpartitioned cookies are available but not in use.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(cross_site);
    addCommonCleanupCallback(t);

    await grantStorageAccessForEmbedSite(t, cross_site);
    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin',
                                              https_origin]]));
    // Retrieve the pre-retry headers.
    const headers = await sendRetrieveRequest(key);
    // Unpartitioned cookie should not be included before the retry.
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['inactive'],
                                      'origin': [https_origin], 'cookie': undefined});
    // Retrieve the headers for the retried request.
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    // The unpartitioned cookie should have been included in the retry.
    assertHeaderValuesMatch(retried_headers, {
    'sec-fetch-storage-access': ['active'],
        'origin': [https_origin],
        'cookie': ['cookie=unpartitioned']
    });
}, "Sec-Fetch-Storage-Access is `active` after a valid retry with matching explicit allowed-origin.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(cross_site);
    addCommonCleanupCallback(t);

    await grantStorageAccessForEmbedSite(t, cross_site);
    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin','*']]));
    // Retrieve the pre-retry headers.
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {
        'sec-fetch-storage-access': ['inactive'],
        'origin': [https_origin],
        'cookie': undefined
    });
    // Retrieve the headers for the retried request.
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assertHeaderValuesMatch(retried_headers, {
        'sec-fetch-storage-access': ['active'],
        'origin': [https_origin],
        'cookie': ['cookie=unpartitioned']
    });
}, "Sec-Fetch-Storage-Access is active after retry with wildcard `allowed-origin` value.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    await grantStorageAccessForEmbedSite(t, cross_site);
    await sendReadableHeaderRequest(
                            makeURL(key, cross_site, retry_path,
                                    [['retry-allowed-origin', '']]));

    // The server behavior when retrieving a header that was never sent is
    // indistinguishable from its behavior when retrieving a header that was
    // sent but was previously retrieved. To ensure the request to retrieve the
    // post-retry header occurs only because they were never sent, always
    // test its retrieval first.
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assert_equals(retried_headers, undefined);
    // Retrieve the pre-retry headers.
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['inactive']});
}, "'Activate-Storage-Access: retry' is a no-op on a request without an `allowed-origin` value.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    await grantStorageAccessForEmbedSite(t, cross_site);
    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin',
                                              same_site]]));
    // Should not be able to retrieve any headers at the post-retry key.
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assert_equals(retried_headers, undefined);
    // Retrieve the pre-retry headers.
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['inactive']});
}, "'Activate-Storage-Access: retry' is a no-op on a request from an origin that does not match its `allowed-origin` value.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin',
                                              https_origin]]));
    // Should not be able to retrieve any headers at the post-retry key.
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assert_equals(retried_headers, undefined);
    // Retrieve the pre-retry headers.
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['none'],
                                      'origin': undefined});
}, "Activate-Storage-Access `retry` is a no-op on a request with a `none` Storage Access status.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    await grantStorageAccessForEmbedSite(t, cross_site);
    const load_header_iframe = await CreateFrame(makeURL(key, cross_site,
                                       non_retry_path,
                                       [['load', ''],
                                        ['script', responder_script]]));
    assert_true(await FrameHasStorageAccess(load_header_iframe),
                "frame should have storage access because of the `load` header");
}, "Activate-Storage-Access `load` header grants storage access to frame.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    const iframe = await CreateFrame(makeURL(key, cross_site,
                                       non_retry_path,
                                       [['script', responder_script]]));
    t.add_cleanup(async () => {
        SetPermissionInFrame(iframe,
            [{ name: 'storage-access' }, 'prompt']);
    });
    await SetPermissionInFrame(iframe,
                [{ name: 'storage-access' }, 'granted']);
    await RequestStorageAccessInFrame(iframe);
    // Create a child iframe with the same source, that causes the server to
    // respond with the `load` header.
    const nested_iframe = await CreateFrameHelper((frame) => {
        // Need a unique `key` on the request or else the server will fail it.
        frame.src = makeURL(key + 'load', cross_site, non_retry_path,
                            [['load', ''], ['script', responder_script]]);
        iframe.appendChild(frame);
      }, false);
    // The nested frame will have storage access because of the `load` response.
    assert_true(await FrameHasStorageAccess(nested_iframe));
}, "Activate-Storage-Access `load` is honored for `active` cases.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    addCommonCleanupCallback(t);

    const load_header_iframe = await CreateFrame(makeURL(key, cross_site,
                                       non_retry_path,
                                       [['load', ''],
                                        ['script', responder_script]]));
    assert_false(await FrameHasStorageAccess(load_header_iframe),
                "frame should not have received storage access.");
}, "Activate-Storage-Access `load` header is a no-op for requests without storage access.");

promise_test(async t => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess('*', '*', 'blocked');
    addCommonCleanupCallback(t);

    const iframe_params = new URLSearchParams([['script',
                                                'embedded_responder.js']]);
    const iframe = await CreateFrame(cross_site + nested_path + '?' +
                                     iframe_params.toString());

    // Create a cross-site request within the iframe
    const nested_url_params = new URLSearchParams([['key', key]]);
    const nested_url = https_origin + non_retry_path + '?' +
                       nested_url_params.toString();
    await NoCorsFetchFromFrame(iframe, nested_url);

    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['inactive'],
                                      'origin': [cross_site]});
}, "Sec-Fetch-Storage-Access is `inactive` for ABA case.");

promise_test(async t => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess('*', '*', 'blocked');
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(https_origin);
    addCommonCleanupCallback(t);

    const iframe_params = new URLSearchParams([['script',
                                                'embedded_responder.js']]);
    const iframe = await CreateFrame(cross_site + nested_path + '?' +
                                     iframe_params.toString());

    const nested_url_params = new URLSearchParams([
                                    ['key', key],
                                    ['retry-allowed-origin', cross_site]]);
    const nested_url = https_origin + retry_path +
                       '?' + nested_url_params.toString();
    await NoCorsFetchFromFrame(iframe, nested_url);
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {
        'sec-fetch-storage-access': ['inactive'],
        'origin': [cross_site],
        'cookie': undefined
    });

    // Storage access should have been activated, without the need for a grant,
    // on the ABA case.
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assertHeaderValuesMatch(retried_headers, {
        'sec-fetch-storage-access': ['active'],
        'origin': [cross_site],
        'cookie': ['cookie=unpartitioned']
    });
}, "Storage Access can be activated for ABA cases by retrying.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(cross_site);
    addCommonCleanupCallback(t);

    await grantStorageAccessForEmbedSite(t, cross_site);

    // Create a redirect destination that is same origin to the initial
    // request.
    const redirect_url = makeURL(key,
                                 cross_site,
                                 retry_path,
                                 [['redirected', '']]);
    // Send a request instructing the server include the `retry` response,
    // and then redirect when storage access has been activated.
    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin',
                                                https_origin],
                                             ['once-active-redirect-location',
                                                redirect_url]]));
    // Confirm the normal retry behavior.
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['inactive'],
                                      'origin': [https_origin], 'cookie': undefined});
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assertHeaderValuesMatch(retried_headers, {
        'sec-fetch-storage-access': ['active'],
        'origin': [https_origin],
        'cookie': ['cookie=unpartitioned']
    });
    // Retrieve the headers for the post-retry redirect request.
    const redirected_headers = await sendRetrieveRequest(redirectedKey(retriedKey(key)));
    assertHeaderValuesMatch(redirected_headers, {
        'sec-fetch-storage-access': ['active'],
        'origin': [https_origin],
        'cookie': ['cookie=unpartitioned']
    });
}, "Sec-Fetch-Storage-Access maintains value on same-origin redirect.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(cross_site);
    addCommonCleanupCallback(t);

    await grantStorageAccessForEmbedSite(t, cross_site);

    // Create a redirect destination that is cross-origin same-site to the
    // initial request.
    const redirect_url = makeURL(key, alt_cross_site, retry_path,
                                 [['redirected', '']]);
    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin',
                                                https_origin],
                                             ['once-active-redirect-location',
                                                redirect_url]]));

    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {
        'sec-fetch-storage-access': ['inactive'],
        'origin': [https_origin],
        'cookie': undefined
    });
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assertHeaderValuesMatch(retried_headers, {
        'sec-fetch-storage-access': ['active'],
        'origin': [https_origin],
        'cookie': ['cookie=unpartitioned']
    });
    // Retrieve the headers for the post-retry redirect request.
    const redirected_headers = await sendRetrieveRequest(redirectedKey(key));
    assertHeaderValuesMatch(redirected_headers, {
        'sec-fetch-storage-access': ['inactive'],
        'origin': ['null'],
        'cookie': undefined
    });
}, "Sec-Fetch-Storage-Access is not 'active' after cross-origin same-site redirection.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(cross_site);
    addCommonCleanupCallback(t);
    await grantStorageAccessForEmbedSite(t, cross_site);

    // Create a redirect destination that is cross-site to the
    // initial request.
    const redirect_url = makeURL(key, https_origin, retry_path,
                                 [['redirected', '']]);
    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin',
                                                https_origin],
                                             ['once-active-redirect-location',
                                                redirect_url]]));

    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {
        'sec-fetch-storage-access': ['inactive'],
        'origin': [https_origin],
        'cookie': undefined
    });
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assertHeaderValuesMatch(retried_headers, {
        'sec-fetch-storage-access': ['active'],
        'origin': [https_origin],
        'cookie': ['cookie=unpartitioned']
    });
    // Retrieve the headers for the post-retry redirect request.
    const redirected_headers = await sendRetrieveRequest(redirectedKey(key));
    // These values will be empty because the frame is now same origin with
    // the top level.
    assertHeaderValuesMatch(redirected_headers, {
        'sec-fetch-storage-access': undefined,
        'origin': ['null'],
        'cookie': undefined
    });
}, "Sec-Fetch-Storage-Access loses value on a cross-site redirection.");

promise_test(async (t) => {
    const key = '{{uuid()}}';
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(cross_site);
    addCommonCleanupCallback(t);
    await grantStorageAccessForEmbedSite(t, cross_site);

    // Create a redirect destination that is cross-origin same-site to the
    // initial request.
    const redirect_url = makeURL(key, https_origin, retry_path, [['redirected', '']]);
     // Send a request that instructs the server to respond with both retry and
     // response headers.
    await sendReadableHeaderRequest(makeURL(key, cross_site, retry_path,
                                            [['retry-allowed-origin',
                                                https_origin],
                                             ['redirect-location',
                                                redirect_url]]));
    // No redirect should have occurred, so a retrieval request for the
    // redirect request should fail.
    const redirected_headers = await sendRetrieveRequest(redirectedKey(key));
    assert_equals(redirected_headers, undefined);
    // Confirm the normal retry behavior.
    const headers = await sendRetrieveRequest(key);
    assertHeaderValuesMatch(headers, {'sec-fetch-storage-access': ['inactive'],
                                      'origin': [https_origin],
                                      'cookie': undefined});
    const retried_headers = await sendRetrieveRequest(retriedKey(key));
    assertHeaderValuesMatch(retried_headers, {
        'sec-fetch-storage-access': ['active'],
        'origin': [https_origin],
        'cookie': ['cookie=unpartitioned']
    });
}, "Activate-Storage-Access retry is handled before any redirects are followed.");