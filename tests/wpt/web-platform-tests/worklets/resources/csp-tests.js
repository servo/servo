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
  const worklet = get_worklet(workletType);

  promise_test(t => {
    const kWindowURL =
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, script-src \'self\' \'unsafe-inline\')';
    const kScriptURL =
        get_host_info().HTTPS_REMOTE_ORIGIN +
        '/worklets/resources/import-empty-worklet-script.js' +
        '?pipe=header(Access-Control-Allow-Origin, *)';
    return openWindowAndExpectResult(
        kWindowURL, kScriptURL, workletType, 'REJECTED');
  }, 'Importing a remote-origin worklet script should be blocked by the ' +
     'script-src \'self\' directive.');

  promise_test(t => {
    const kWindowURL =
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, script-src \'self\' \'unsafe-inline\')';
    const kScriptURL = 'import-remote-origin-empty-worklet-script.sub.js';
    return openWindowAndExpectResult(
        kWindowURL, kScriptURL, workletType, 'REJECTED');
  }, 'Importing a remote-origin script from a same-origin worklet script ' +
     'should be blocked by the script-src \'self\' directive.');

  promise_test(t => {
    const kWindowURL =
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, script-src * \'unsafe-inline\')';
    const kScriptURL = get_host_info().HTTPS_REMOTE_ORIGIN +
                       '/worklets/resources/empty-worklet-script.js' +
                       '?pipe=header(Access-Control-Allow-Origin, *)';
    return openWindowAndExpectResult(
        kWindowURL, kScriptURL, workletType, 'RESOLVED');
  }, 'Importing a remote-origin worklet script should not be blocked ' +
     'because the script-src * directive allows it.');

  promise_test(t => {
    const kWindowURL =
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, script-src * \'unsafe-inline\')';
    // A worklet on HTTPS_REMOTE_ORIGIN will import a child script on
    // HTTPS_REMOTE_ORIGIN.
    const kScriptURL = get_host_info().HTTPS_REMOTE_ORIGIN +
                       '/worklets/resources/import-empty-worklet-script.js' +
                       '?pipe=header(Access-Control-Allow-Origin, *)';
    return openWindowAndExpectResult(
        kWindowURL, kScriptURL, workletType, 'RESOLVED');
  }, 'Importing a remote-origin script from a remote-origin worklet script '+
     'should not be blocked because the script-src * directive allows it.');

  promise_test(t => {
    const kWindowURL =
        'resources/addmodule-window.html?pipe=header(' +
        'Content-Security-Policy, worker-src \'self\' \'unsafe-inline\')';
    const kScriptURL = get_host_info().HTTPS_REMOTE_ORIGIN +
                       '/worklets/resources/empty-worklet-script.js' +
                       '?pipe=header(Access-Control-Allow-Origin, *)';
    return openWindowAndExpectResult(
        kWindowURL, kScriptURL, workletType, 'RESOLVED');
  }, 'Importing a remote-origin worklet script should not be blocked by ' +
     'the worker-src directive because worklets obey the script-src ' +
     'directive.');
}
