function createCookieValue(settings) {
  return settings.credentials + '-' + settings.origin;
}

function createSetCookieURL(settings) {
  const params = new URLSearchParams;
  params.append('name', 'cookieName');
  params.append('value', createCookieValue(settings));
  if (settings.origin == 'same') {
    return get_host_info().HTTPS_ORIGIN +
           '/worklets/resources/set-cookie.py?' + params;
  }
  if (settings.origin == 'remote') {
    return get_host_info().HTTPS_REMOTE_ORIGIN +
           '/worklets/resources/set-cookie.py?' + params;
  }
  assert_unreached('settings.origin has an invalid value.');
}

function createScriptURL(settings) {
  const params = new URLSearchParams;
  if (settings.expectCredentialsSent)
    params.append('value', createCookieValue(settings));
  if (settings.origin == 'same') {
    return get_host_info().HTTPS_ORIGIN +
           '/worklets/resources/credentials.py?' + params;
  }
  if (settings.origin == 'remote') {
    return get_host_info().HTTPS_REMOTE_ORIGIN +
           '/worklets/resources/credentials.py?' + params;
  }
  assert_unreached('settings.origin has an invalid value.');
}

function createWorkletOptions(settings) {
  if (settings.credentials == '')
    return {};
  return { credentials: settings.credentials };
}

// Run a credentials test with the given settings.
//
// Example:
// settings = {
//   workletType: 'paint',
//   credentials: 'include',
//   origin: 'same',  // 'same' or 'remote'
//   expectCredentialsSent: true
// };
function runCredentialsTest(settings) {
  const worklet = get_worklet(settings.workletType);
  const setCookieURL = createSetCookieURL(settings);
  const scriptURL = createScriptURL(settings);
  const options = createWorkletOptions(settings);

  // { credentials: 'include' } is necessary for configuring document's cookies
  // with the Set-Cookie: header of the response.
  return fetch(setCookieURL, { mode: 'cors', credentials: 'include' })
      .then(response => worklet.addModule(scriptURL, options));
}

// Runs a series of tests related to credentials on a worklet.
//
// Usage:
// runCredentialsTests("paint");
function runCredentialsTests(worklet_type) {
  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: '',
                                origin: 'same',
                                expectCredentialsSent: false });
  }, 'Importing a same-origin script with the default WorkletOptions should ' +
     'not send the credentials');

  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: '',
                                origin: 'remote',
                                expectCredentialsSent: false });
  }, 'Importing a remote-origin script with the default WorkletOptions ' +
     'should not send the credentials');

  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: 'omit',
                                origin: 'same',
                                expectCredentialsSent: false });
  }, 'Importing a same-origin script with credentials=omit should not send ' +
     'the credentials');

  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: 'omit',
                                origin: 'remote',
                                expectCredentialsSent: false });
  }, 'Importing a remote-origin script with credentials=omit should not send ' +
     'the credentials');

  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: 'same-origin',
                                origin: 'same',
                                expectCredentialsSent: true });
  }, 'Importing a same-origin script with credentials=same-origin should ' +
     'send the credentials');

  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: 'same-origin',
                                origin: 'remote',
                                expectCredentialsSent: false });
  }, 'Importing a remote-origin script with credentials=same-origin should ' +
     'not send the credentials');

  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: 'include',
                                origin: 'same',
                                expectCredentialsSent: true });
  }, 'Importing a same-origin script with credentials=include should send ' +
     'the credentials');

  promise_test(() => {
    return runCredentialsTest({ workletType: worklet_type,
                                credentials: 'include',
                                origin: 'remote',
                                expectCredentialsSent: true });
  }, 'Importing a remote-origin script with credentials=include should ' +
     'send the credentials');
}
