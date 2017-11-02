function openWindow(url) {
  return new Promise(resolve => {
      let win = window.open(url, '_blank');
      add_completion_callback(() => win.close());
      window.onmessage = e => {
        assert_equals(e.data, 'LOADED');
        resolve(win);
      };
    });
}

// Runs a series of tests related to the referrer policy on a worklet.
//
// Usage:
// runReferrerTests("paint");
function runReferrerTests(worklet_type) {
  const worklet = get_worklet(worklet_type);

  promise_test(() => {
      const kWindowURL = "resources/referrer-window.html" +
                         "?pipe=header(Referrer-Policy,no-referrer)";
      return openWindow(kWindowURL).then(win => {
          const promise = new Promise(resolve => window.onmessage = resolve);
          win.postMessage({ type: worklet_type,
                            referrer_policy: 'no-referrer',
                            is_cross_origin: false }, '*');
          return promise;
      }).then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
  }, 'Importing a same-origin script from a page that has "no-referrer" ' +
     'referrer policy should not send referrer.');

  promise_test(() => {
      const kWindowURL = "resources/referrer-window.html" +
                         "?pipe=header(Referrer-Policy,no-referrer)";
      return openWindow(kWindowURL).then(win => {
          const promise = new Promise(resolve => window.onmessage = resolve);
          win.postMessage({ type: worklet_type,
                            referrer_policy: 'no-referrer',
                            is_cross_origin: true }, '*');
          return promise;
      }).then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
  }, 'Importing a remote-origin script from a page that has "no-referrer" ' +
     'referrer policy should not send referrer.');

  promise_test(() => {
      const kWindowURL = 'resources/referrer-window.html' +
                         '?pipe=header(Referrer-Policy,origin)';
      return openWindow(kWindowURL).then(win => {
          const promise = new Promise(resolve => window.onmessage = resolve);
          win.postMessage({ type: worklet_type,
                            referrer_policy: 'origin',
                            is_cross_origin: false }, '*');
          return promise;
      }).then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
  }, 'Importing a same-origin script from a page that has "origin" ' +
     'referrer policy should send only an origin as referrer.');

  promise_test(() => {
      const kWindowURL = 'resources/referrer-window.html' +
                         '?pipe=header(Referrer-Policy,origin)';
      return openWindow(kWindowURL).then(win => {
          const promise = new Promise(resolve => window.onmessage = resolve);
          win.postMessage({ type: worklet_type,
                            referrer_policy: 'origin',
                            is_cross_origin: true }, '*');
          return promise;
      }).then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
  }, 'Importing a remote-origin script from a page that has "origin" ' +
     'referrer policy should send only an origin as referrer.');

  promise_test(() => {
      const kWindowURL = 'resources/referrer-window.html' +
                         '?pipe=header(Referrer-Policy,same-origin)';
      return openWindow(kWindowURL).then(win => {
          const promise = new Promise(resolve => window.onmessage = resolve);
          win.postMessage({ type: worklet_type,
                            referrer_policy: 'same-origin',
                            is_cross_origin: false }, '*');
          return promise;
      }).then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
  }, 'Importing a same-origin script from a page that has "same-origin" ' +
     'referrer policy should send referrer.');

  promise_test(() => {
      const kWindowURL = 'resources/referrer-window.html' +
                         '?pipe=header(Referrer-Policy,same-origin)';
      return openWindow(kWindowURL).then(win => {
          const promise = new Promise(resolve => window.onmessage = resolve);
          win.postMessage({ type: worklet_type,
                            referrer_policy: 'same-origin',
                            is_cross_origin: true }, '*');
          return promise;
      }).then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
  }, 'Importing a remote-origin script from a page that has "same-origin" ' +
     'referrer policy should not send referrer.');
}
