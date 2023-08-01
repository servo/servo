// META: script=/storage-access-api/helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

const requestedOrigin = 'https://foo.com';
const altOrigin = 'https://{{hosts[alt][www]}}:{{ports[https][0]}}';

promise_test(
    async () => {
      assert_not_equals(document.requestStorageAccessFor, undefined);
    },
    '[top-level-context] document.requestStorageAccessFor() should be supported on the document interface');

promise_test(
  t => {
    return promise_rejects_js(t, TypeError,
      document.requestStorageAccessFor(),
      'document.requestStorageAccessFor() call without origin argument');
  },
  '[top-level-context] document.requestStorageAccessFor() should be rejected when called with no argument');

// Most tests need to start with the feature in "prompt" state.
// For tests that rely on the permission state, this function is intended to be
// run prior to executing test logic, rather than using clean-up functions for
// the permission.
async function CommonSetup() {
  await test_driver.set_permission(
    { name: 'top-level-storage-access', requestedOrigin }, 'prompt');
  await test_driver.set_permission(
    { name: 'top-level-storage-access', requestedOrigin: altOrigin }, 'prompt');
}

promise_test(async t => {
      await CommonSetup();
      return promise_rejects_dom(t, 'NotAllowedError',
        document.requestStorageAccessFor(requestedOrigin),
        'document.requestStorageAccessFor() call without user gesture');
    },
    '[top-level-context] document.requestStorageAccessFor() should be rejected by default with no user gesture');

promise_test(async t => {
  const description =
      'document.requestStorageAccessFor() call in a detached frame';
  // Can't use promise_rejects_dom here because the exception is from the wrong global.
  return CreateDetachedFrame().requestStorageAccessFor(requestedOrigin)
      .then(t.unreached_func('Should have rejected: ' + description))
      .catch((e) => {
        assert_equals(e.name, 'InvalidStateError', description);
      });
}, '[non-fully-active] document.requestStorageAccessFor() should not resolve when run in a detached frame');

promise_test(async t => {
  const description =
      'document.requestStorageAccessFor() in a detached DOMParser result';
  return CreateDocumentViaDOMParser().requestStorageAccessFor(requestedOrigin)
      .then(t.unreached_func('Should have rejected: ' + description))
      .catch((e) => {
        assert_equals(e.name, 'InvalidStateError', description);
      });
}, '[non-fully-active] document.requestStorageAccessFor() should not resolve when run in a detached DOMParser document');

promise_test(
    async t => {
      await CommonSetup();
      await test_driver.set_permission(
          {name: 'top-level-storage-access', requestedOrigin}, 'granted');

      await document.requestStorageAccessFor(requestedOrigin);
    },
    '[top-level-context] document.requestStorageAccessFor() should be resolved without a user gesture with an existing permission');

promise_test(
    async t => {
      await CommonSetup();
      await test_driver.set_permission(
          {name: 'top-level-storage-access', requestedOrigin: altOrigin},
          'granted');

      const frame = await CreateFrame(
        altOrigin + '/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js');

      await RunCallbackWithGesture(() => document.requestStorageAccessFor(altOrigin));
      assert_true(await RequestStorageAccessInFrame(frame));
    },
    '[top-level-context] document.requestStorageAccess() should be resolved without a user gesture after a successful requestStorageAccessFor() call');

promise_test(
    async t => {
      await RunCallbackWithGesture(
        () => document.requestStorageAccessFor(document.location.origin));
    },
    '[top-level-context] document.requestStorageAccessFor() should be resolved when called properly with a user gesture and the same origin');

promise_test(
    async t => {
      await RunCallbackWithGesture(
        () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessFor('bogus-url'),
          'document.requestStorageAccessFor() call with bogus URL'));
    },
    '[top-level-context] document.requestStorageAccessFor() should be rejected when called with an invalid origin');

promise_test(
    async t => {
      await RunCallbackWithGesture(
        () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessFor('data:,Hello%2C%20World%21'),
          'document.requestStorageAccessFor() call with data URL'));
    },
    '[top-level-context] document.requestStorageAccessFor() should be rejected when called with an opaque origin');

promise_test(
    async (t) => {
      const altEchoCookieHeaderUrl =
          `${altOrigin}/storage-access-api/resources/echo-cookie-header.py`;

      await MaybeSetStorageAccess('*', '*', 'blocked');
      await CommonSetup();

      await test_driver.set_permission(
          {name: 'top-level-storage-access', requestedOrigin: altOrigin},
          'granted');

      // Set cross-site cookie for altOrigin. Note that this only works with
      // an existing top-level storage access permission.
      await fetch(
          `${altOrigin}/cookies/resources/set-cookie.py?name=cookie&path=/&samesite=None&secure=`,
          {mode: 'cors', credentials: 'include'});

      const httpCookies1 = await fetch(altEchoCookieHeaderUrl, {
                              mode: 'cors',
                              credentials: 'include'
                            }).then((resp) => resp.text());
      assert_true(
          httpCookies1.includes('cookie=1'),
          'After obtaining top-level storage access, cross-site subresource requests with CORS mode should have cookie access.');

      const httpCookies2 = await fetch(altEchoCookieHeaderUrl, {
                              mode: 'no-cors',
                              credentials: 'include'
                            }).then((resp) => resp.text());
      assert_false(
          httpCookies2.includes('cookie=1'),
          'Cross-site subresource requests without CORS mode cannot access cookie even with an existing permission.');
    },
    '[top-level-context] Top-level storage access only allows cross-site subresource requests to access cookie when using CORS mode.');

promise_test(
    async () => {
      const frame = await CreateFrame(
        '/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js');
      assert_not_equals(frame.contentWindow.document.requestStorageAccessFor, undefined);
    },
    '[same-origin-iframe] document.requestStorageAccessFor() should be supported on the document interface');

promise_test(
    async t => {
      const frame = await CreateFrame(
        '/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js');
      return promise_rejects_js(t, frame.contentWindow.TypeError,
        frame.contentWindow.document.requestStorageAccessFor(),
        'document.requestStorageAccessFor() call without origin argument');
    },
    '[same-origin-iframe] document.requestStorageAccessFor() should be rejected when called with no argument');

promise_test(
    async t => {
      const frame = await CreateFrame(
        '/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js');

      await RunCallbackWithGesture(() =>
          promise_rejects_dom(t, 'NotAllowedError', frame.contentWindow.DOMException,
            frame.contentWindow.document.requestStorageAccessFor(document.location.origin),
            'document.requestStorageAccessFor() call in a non-top-level context'));
    },
    '[same-origin-iframe] document.requestStorageAccessFor() should be rejected when called in an iframe');

promise_test(
    async (t) => {
      await MaybeSetStorageAccess('*', '*', 'blocked');
      await CommonSetup();

      const frame = await CreateFrame(
        `/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js`);

      // Set cross-site cookie for altOrigin. Note that cookie won't be set
      // even with an existing top-level storage access permission in an
      // iframe.
      await FetchFromFrame(frame,
          `${altOrigin}/cookies/resources/set-cookie.py?name=cookie&path=/&samesite=None&secure=`);

      await test_driver.set_permission(
          {name: 'top-level-storage-access', requestedOrigin: altOrigin},
          'granted');

      const httpCookies = await FetchSubresourceCookiesFromFrame(frame, altOrigin);
      assert_false(httpCookies.includes('cookie=1'));
    },
    '[same-origin-iframe] Existing top-level storage access permission should not allow cookie access for the cross-site subresource requests made in a non-top-level context.');
