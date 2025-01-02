function openWindow(url) {
  return new Promise(resolve => {
    const win = window.open(url, '_blank');
    add_result_callback(() => win.close());
    window.onmessage = e => {
      assert_equals(e.data, 'LOADED');
      resolve(win);
    };
  });
}

function openWindowAndExpectResult(windowURL, scriptURL, type, expectation) {
  return openWindow(windowURL).then(win => {
    const promise = new Promise(r => window.onmessage = r);
    win.postMessage({ type: type, script_url: scriptURL }, '*');
    return promise;
  }).then(msg_event => assert_equals(msg_event.data, expectation));
}

// Runs a series of tests related to content security policy on a worklet.
//
// Usage:
// runContentSecurityPolicyTests("paint");
function runContentSecurityPolicyTests(workletType) {
  runSrcTests(workletType);
  runMixedContentTests(workletType);
}

// script-src and worker-src tests.
function runSrcTests(workletType) {
  const kWindowConfigs = [
    {
      'windowURL':
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, script-src \'self\' \'unsafe-inline\')',
      'crossOriginExpectation': 'REJECTED',
      'message': 'should be blocked by the script-src \'self\' directive.'
    },
    {
      'windowURL':
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, script-src ' + location.origin + ' ' +
        get_host_info().HTTPS_REMOTE_ORIGIN + ' \'unsafe-inline\')',
      'crossOriginExpectation': 'RESOLVED',
      'message':
        'should not be blocked because the script-src directive ' +
        'specifying the origin allows it.'
    },
    {
      'windowURL':
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, script-src * \'unsafe-inline\')',
      'crossOriginExpectation': 'RESOLVED',
      'message':
        'should not be blocked because the script-src * directive allows it.'
    },
    {
      'windowURL':
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, worker-src \'self\' \'unsafe-inline\')',
      'crossOriginExpectation': 'RESOLVED',
      'message':
        'should not be blocked by the worker-src directive ' +
        'because worklets obey the script-src directive.'
    }
  ];
  for (const windowConfig of kWindowConfigs) {
    promise_test(t => {
        const kScriptURL =
          get_host_info().HTTPS_REMOTE_ORIGIN +
          '/worklets/resources/empty-worklet-script-with-cors-header.js';
        return openWindowAndExpectResult(
          windowConfig.windowURL, kScriptURL, workletType,
          windowConfig.crossOriginExpectation);
      },
      'A remote-origin worklet ' + windowConfig.message);

    promise_test(t => {
        const kScriptURL = 'import-remote-origin-empty-worklet-script.sub.js';
        return openWindowAndExpectResult(
          windowConfig.windowURL, kScriptURL, workletType,
          windowConfig.crossOriginExpectation);
      },
      'A same-origin worklet importing a remote-origin script ' +
      windowConfig.message);

    promise_test(t => {
        // A worklet on HTTPS_REMOTE_ORIGIN will import a child script on
        // HTTPS_REMOTE_ORIGIN.
        const kScriptURL =
          get_host_info().HTTPS_REMOTE_ORIGIN +
          '/worklets/resources/import-empty-worklet-script-with-cors-header.js';
        return openWindowAndExpectResult(
          windowConfig.windowURL, kScriptURL, workletType,
          windowConfig.crossOriginExpectation);
      },
      'A remote-origin worklet importing a remote-origin script ' +
      windowConfig.message);

    promise_test(t => {
        const kScriptURL =
          '/common/redirect.py?location=' + encodeURIComponent(
              get_host_info().HTTPS_REMOTE_ORIGIN +
              '/worklets/resources/empty-worklet-script-with-cors-header.js');
        return openWindowAndExpectResult(
          windowConfig.windowURL, kScriptURL, workletType,
          windowConfig.crossOriginExpectation);
      },
      'A remote-origin-redirected worklet ' + windowConfig.message);

    promise_test(t => {
        const kScriptURL =
          'import-remote-origin-redirected-empty-worklet-script.sub.js';
        return openWindowAndExpectResult(
          windowConfig.windowURL, kScriptURL, workletType,
          windowConfig.crossOriginExpectation);
      },
      'A same-origin worklet importing a remote-origin-redirected script ' +
      windowConfig.message);
  }
}

// Mixed content tests.
function runMixedContentTests(workletType) {
  const kInsecureURL =
      get_host_info().HTTP_ORIGIN +
      '/worklets/resources/empty-worklet-script-with-cors-header.js';
  const kScriptConfigs = [
    {URL: kInsecureURL,
     message: 'An insecure-origin worklet'},
    {URL: '/common/redirect.py?location=' + encodeURIComponent(kInsecureURL),
     message: 'An insecure-origin-redirected worklet'},
    {URL: 'import-insecure-origin-empty-worklet-script.sub.js',
     message: 'A same-origin worklet importing an insecure-origin script'},
    {URL: 'import-insecure-origin-redirected-empty-worklet-script.sub.js',
     message: 'A same-origin worklet ' +
              'importing an insecure-origin-redirected script'}
  ];
  for (const scriptConfig of kScriptConfigs) {
    promise_test(t => {
        const kWindowURL = 'resources/addmodule-window.html';
        return openWindowAndExpectResult(
          kWindowURL, scriptConfig.URL, workletType, 'REJECTED');
      },
      scriptConfig.message + ' should be blocked because of mixed contents.');
  }
}
