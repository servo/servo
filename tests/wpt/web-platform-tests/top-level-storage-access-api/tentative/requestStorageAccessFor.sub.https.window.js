// META: script=/storage-access-api/helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// Note that this file follows the pattern in:
// storage-access-api/requestStorageAccess.sub.window.js
//
// Some tests are run at the top-level, and an iframe is added to validate API
// behavior in that context.

// Prefix each test case with an indicator so we know what context they are run
// in if they are used in multiple iframes.
let testPrefix = 'top-level-context';

// Keep track of if we run these tests in a nested context, we don't want to
// recurse forever.
let topLevelDocument = true;

// The query string allows derivation of test conditions, like whether the tests
// are running in a top-level context.
const queryParams = window.location.search.substring(1).split('&');
queryParams.forEach((param) => {
  if (param.toLowerCase() == 'rootdocument=false') {
    topLevelDocument = false;
  } else if (param.split('=')[0].toLowerCase() == 'testcase') {
    testPrefix = param.split('=')[1];
  }
});

const requestedOrigin = 'https://foo.com';

// TODO(crbug.com/1351540): when/if requestStorageAccessFor is standardized,
// upstream with the Storage Access API helpers file.
function RunRequestStorageAccessForInDetachedFrame(origin) {
  const nestedFrame = document.createElement('iframe');
  document.body.append(nestedFrame);
  const inner_doc = nestedFrame.contentDocument;
  nestedFrame.remove();
  return inner_doc.requestStorageAccessFor(origin);
}

function RunRequestStorageAccessForViaDomParser(origin) {
  const parser = new DOMParser();
  const doc = parser.parseFromString('<html></html>', 'text/html');
  return doc.requestStorageAccessFor(origin);
}

// Common tests to run in all frames.
test(
    () => {
      assert_not_equals(document.requestStorageAccessFor, undefined);
    },
    '[' + testPrefix +
        '] document.requestStorageAccessFor() should be supported on the document interface');

// Promise tests should all start with the feature in "prompt" state.
promise_setup(async () => {
  await test_driver.set_permission(
    { name: 'top-level-storage-access', requestedOrigin }, 'prompt');
  await test_driver.set_permission({name: 'storage-access'}, 'prompt');
});

promise_test(
  t => {
    return promise_rejects_js(t, TypeError,
      document.requestStorageAccessFor(),
      'document.requestStorageAccessFor() call without origin argument');
  },
  '[' + testPrefix +
      '] document.requestStorageAccessFor() should be rejected when called with no argument');

if (topLevelDocument) {
  promise_test(
      t => {
        return promise_rejects_dom(t, 'NotAllowedError',
          document.requestStorageAccessFor(requestedOrigin),
         'document.requestStorageAccessFor() call without user gesture');
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be rejected by default with no user gesture');

  promise_test(async t => {
    const description =
        'document.requestStorageAccessFor() call in a detached frame';
    // Can't use promise_rejects_dom here because the exception is from the wrong global.
    return RunRequestStorageAccessForInDetachedFrame(requestedOrigin)
        .then(t.unreached_func('Should have rejected: ' + description))
        .catch((e) => {
          assert_equals(e.name, 'InvalidStateError', description);
        });
  }, '[non-fully-active] document.requestStorageAccessFor() should not resolve when run in a detached frame');

  promise_test(async t => {
    const description =
        'document.requestStorageAccessFor() in a detached DOMParser result';
    return RunRequestStorageAccessForViaDomParser(requestedOrigin)
        .then(t.unreached_func('Should have rejected: ' + description))
        .catch((e) => {
          assert_equals(e.name, 'InvalidStateError', description);
        });
  }, '[non-fully-active] document.requestStorageAccessFor() should not resolve when run in a detached DOMParser document');

  promise_test(
    async t => {
      await test_driver.set_permission(
        { name: 'top-level-storage-access', requestedOrigin }, 'granted');

      await document.requestStorageAccessFor(requestedOrigin);
    },
    '[' + testPrefix +
    '] document.requestStorageAccessFor() should be resolved without a user gesture with an existing permission');

  promise_test(
      async t => {
        const altOrigin = 'https://{{hosts[alt][www]}}:{{ports[https][0]}}';
        t.add_cleanup(async () => {
          await test_driver.set_permission(
              {name: 'top-level-storage-access', requestedOrigin: altOrigin},
              'prompt');
          await test_driver.set_permission({name: 'storage-access'}, 'prompt');
        });
        await test_driver.set_permission(
            {name: 'top-level-storage-access', requestedOrigin: altOrigin},
            'granted');

        await RunCallbackWithGesture(() => {
          document.requestStorageAccessFor(altOrigin).then(() => {
            RunTestsInIFrame(
                'https://{{hosts[alt][www]}}:{{ports[https][0]}}/top-level-storage-access-api/tentative/resources/requestStorageAccess-integration-iframe.https.html');
          });
        });
      },
      '[' + testPrefix +
          '] document.requestStorageAccess() should be resolved without a user gesture after a successful requestStorageAccessFor() call');

  // Create a test with a single-child same-origin iframe.
  // This will validate that calls to requestStorageAccessFor are rejected
  // in non-top-level contexts.
  RunTestsInIFrame(
      './resources/requestStorageAccessFor-iframe.https.html?testCase=same-origin-frame&rootdocument=false');

  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => document.requestStorageAccessFor(document.location.origin));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be resolved when called properly with a user gesture and the same site');

  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessFor('bogus-url'),
            'document.requestStorageAccessFor() call with bogus URL'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be rejected when called with an invalid site');

  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessFor('data:,Hello%2C%20World%21'),
            'document.requestStorageAccessFor() call with data URL'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be rejected when called with an opaque origin');

  promise_test(
      async (t) => {
        const altOrigin = 'https://{{hosts[alt][www]}}:{{ports[https][0]}}';
        const altEchoCookieHeaderUrl =
            `${altOrigin}/storage-access-api/resources/echo-cookie-header.py`;

        await MaybeSetStorageAccess('*', '*', 'blocked');
        t.add_cleanup(async () => {
          await test_driver.delete_all_cookies();
          await test_driver.set_permission(
              {name: 'top-level-storage-access', requestedOrigin: altOrigin},
              'prompt');
          await MaybeSetStorageAccess('*', '*', 'allowed');
        });

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
      '[' + testPrefix +
          '] Top-level storage access only allows cross-site subresource requests to access cookie when using CORS mode.');

} else {
  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessFor(document.location.origin),
            'document.requestStorageAccessFor() call in a non-top-level context'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be rejected when called in an iframe');

  promise_test(
      async (t) => {
        const altOrigin = 'https://{{hosts[alt][www]}}:{{ports[https][0]}}';

        await MaybeSetStorageAccess('*', '*', 'blocked');
        t.add_cleanup(async () => {
          await test_driver.delete_all_cookies();
          await test_driver.set_permission(
              {name: 'top-level-storage-access', requestedOrigin: altOrigin},
              'prompt');
          await MaybeSetStorageAccess('*', '*', 'allowed');
        });

        // Set cross-site cookie for altOrigin. Note that cookie won't be set
        // even with an existing top-level storage access permission in an
        // iframe.
        await fetch(
            `${altOrigin}/cookies/resources/set-cookie.py?name=cookie&path=/&samesite=None&secure=`,
            {mode: 'cors', credentials: 'include'});

        await test_driver.set_permission(
            {name: 'top-level-storage-access', requestedOrigin: altOrigin},
            'granted');

        const httpCookies =
            await fetch(
                `${altOrigin}/storage-access-api/resources/echo-cookie-header.py`,
                {mode: 'cors', credentials: 'include'})
                .then((resp) => resp.text());
        assert_false(httpCookies.includes('cookie=1'));
      },
      '[' + testPrefix +
          '] Existing top-level storage access permission should not allow cookie access for the cross-site subresource requests made in a non-top-level context.');
}
