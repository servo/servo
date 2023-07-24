function openWindow(t, url) {
  return new Promise(resolve => {
      let win = window.open(url, '_blank');
      t.add_cleanup(() => win.close());
      window.onmessage = e => {
        assert_equals(e.data, 'LOADED');
        resolve(win);
      };
    });
}

// Run a referrer policy test with the given settings.
//
// Example:
// settings = {
//   workletType: 'paint',
//   fetchType: 'top-level' or 'descendant',
//   referrerPolicy: 'no-referrer',
//   scriptsOrigins: { topLevel: 'same', descendant: 'remote' }
// };
function runReferrerTest(t, settings) {
  const kWindowURL =
      'resources/referrer-window.html' +
      `?pipe=header(Referrer-Policy,${settings.referrerPolicy})`;
  return openWindow(t, kWindowURL).then(win => {
    const promise = new Promise(resolve => window.onmessage = resolve);
    win.postMessage(settings, '*');
    return promise;
  }).then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
}

// Runs a series of tests related to the referrer policy on a worklet.
//
// Usage:
// runReferrerTests("paint");
function runReferrerTests(workletType) {
  const worklet = get_worklet(workletType);

  // Tests for top-level script fetch -----------------------------------------

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'top-level',
                                referrerPolicy: 'no-referrer',
                                scriptOrigins: { topLevel: 'same' } });
  }, 'Importing a same-origin script from a page that has "no-referrer" ' +
     'referrer policy should not send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'top-level',
                                referrerPolicy: 'no-referrer',
                                scriptOrigins: { topLevel: 'remote' } });
  }, 'Importing a remote-origin script from a page that has "no-referrer" ' +
     'referrer policy should not send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'top-level',
                                referrerPolicy: 'origin',
                                scriptOrigins: { topLevel: 'same' } });
  }, 'Importing a same-origin script from a page that has "origin" ' +
     'referrer policy should send only an origin as referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'top-level',
                                referrerPolicy: 'origin',
                                scriptOrigins: { topLevel: 'remote' } });
  }, 'Importing a remote-origin script from a page that has "origin" ' +
     'referrer policy should send only an origin as referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'top-level',
                                referrerPolicy: 'same-origin',
                                scriptOrigins: { topLevel: 'same' } });
  }, 'Importing a same-origin script from a page that has "same-origin" ' +
     'referrer policy should send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'top-level',
                                referrerPolicy: 'same-origin',
                                scriptOrigins: { topLevel: 'remote' } });
  }, 'Importing a remote-origin script from a page that has "same-origin" ' +
     'referrer policy should not send referrer.');

  // Tests for descendant script fetch -----------------------------------------

  // Referrer policy: no-referrer.
  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'no-referrer',
                                scriptOrigins: { topLevel: 'same',
                                                 descendant: 'same' } });
  }, 'Importing a same-origin script from a same-origin worklet script that ' +
     'has "no-referrer" referrer policy should not send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'no-referrer',
                                scriptOrigins: { topLevel: 'same',
                                                 descendant: 'remote' } });
  }, 'Importing a remote-origin script from a same-origin worklet script ' +
     'that has "no-referrer" referrer policy should not send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'no-referrer',
                                scriptOrigins: { topLevel: 'remote',
                                                 descendant: 'same' } });
  }, 'Importing a same-origin script from a remote-origin worklet script ' +
     'that has "no-referrer" referrer policy should not send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'no-referrer',
                                scriptOrigins: { topLevel: 'remote',
                                                 descendant: 'remote' } });
  }, 'Importing a remote-origin script from a remote-origin worklet script ' +
     'that has "no-referrer" referrer policy should not send referrer.');

  // Referrer policy: origin.
  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'origin',
                                scriptOrigins: { topLevel: 'same',
                                                 descendant: 'same' } });
  }, 'Importing a same-origin script from a same-origin worklet script that ' +
     'has "origin" referrer policy should send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'origin',
                                scriptOrigins: { topLevel: 'same',
                                                 descendant: 'remote' } });
  }, 'Importing a remote-origin script from a same-origin worklet script ' +
     'that has "origin" referrer policy should send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'origin',
                                scriptOrigins: { topLevel: 'remote',
                                                 descendant: 'same' } });
  }, 'Importing a same-origin script from a remote-origin worklet script ' +
     'that has "origin" referrer policy should send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'origin',
                                scriptOrigins: { topLevel: 'remote',
                                                 descendant: 'remote' } });
  }, 'Importing a remote-origin script from a remote-origin worklet script ' +
     'that has "origin" referrer policy should send referrer.');

  // Referrer policy: same-origin.
  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'same-origin',
                                scriptOrigins: { topLevel: 'same',
                                                 descendant: 'same' } });
  }, 'Importing a same-origin script from a same-origin worklet script that ' +
     'has "same-origin" referrer policy should send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'same-origin',
                                scriptOrigins: { topLevel: 'same',
                                                 descendant: 'remote' } });
  }, 'Importing a remote-origin script from a same-origin worklet script ' +
     'that has "same-origin" referrer policy should not send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'same-origin',
                                scriptOrigins: { topLevel: 'remote',
                                                 descendant: 'same' } });
  }, 'Importing a same-origin script from a remote-origin worklet script ' +
     'that has "same-origin" referrer policy should not send referrer.');

  promise_test(t => {
    return runReferrerTest(t, { workletType: workletType,
                                fetchType: 'descendant',
                                referrerPolicy: 'same-origin',
                                scriptOrigins: { topLevel: 'remote',
                                                 descendant: 'remote' } });
  }, 'Importing a remote-origin script from a remote-origin worklet script ' +
     'that has "same-origin" referrer policy should send referrer.');
}
