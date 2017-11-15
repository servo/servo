// Runs a series of tests related to credentials on a worklet.
//
// Usage:
// runCredentialsTests("paint");
function runCredentialsTests(worklet_type) {
  const worklet = get_worklet(worklet_type);

  promise_test(() => {
      document.cookie = 'cookieName=default';
      const kScriptURL = 'resources/credentials.py?mode=default';
      return worklet.addModule(kScriptURL).then(undefined_arg => {
        assert_equals(undefined_arg, undefined);
      });
  }, 'Importing a same-origin script with the default WorkletOptions should ' +
     'omit the credentials');

  promise_test(() => {
      const kSetCookieURL =
          get_host_info().HTTPS_REMOTE_ORIGIN +
          '/worklets/resources/set-cookie.py?name=cookieName';
      const kScriptURL = get_host_info().HTTPS_REMOTE_ORIGIN +
                         '/worklets/resources/credentials.py?mode=default';
      const kOptions = { credentials: 'same-origin' };

      // Set a cookie in the remote origin and then start a worklet.
      return fetch(kSetCookieURL, { mode: 'cors' })
        .then(() => worklet.addModule(kScriptURL, kOptions))
        .then(undefined_arg => assert_equals(undefined_arg, undefined));
  }, 'Importing a remote-origin script with the default WorkletOptions ' +
     'should not include the credentials');

  promise_test(() => {
      document.cookie = 'cookieName=omit';
      const kScriptURL = 'resources/credentials.py?mode=omit';
      const kOptions = { credentials: 'omit' };
      return worklet.addModule(kScriptURL, kOptions).then(undefined_arg => {
        assert_equals(undefined_arg, undefined);
      });
  }, 'Importing a same-origin script with credentials=omit should omit the ' +
     'credentials');

  promise_test(() => {
      const kSetCookieURL =
          get_host_info().HTTPS_REMOTE_ORIGIN +
          '/worklets/resources/set-cookie.py?name=cookieName';
      const kScriptURL = get_host_info().HTTPS_REMOTE_ORIGIN +
                         '/worklets/resources/credentials.py?mode=omit';
      const kOptions = { credentials: 'omit' };

      // Set a cookie in the remote origin and then start a worklet.
      return fetch(kSetCookieURL, { mode: 'cors' })
        .then(() => worklet.addModule(kScriptURL, kOptions))
        .then(undefined_arg => assert_equals(undefined_arg, undefined));
  }, 'Importing a remote-origin script with credentials=omit should omit the ' +
     'credentials');

  promise_test(() => {
      document.cookie = 'cookieName=same-origin';
      const kScriptURL = 'resources/credentials.py?mode=same-origin';
      const kOptions = { credentials: 'same-origin' };
      return worklet.addModule(kScriptURL, kOptions).then(undefined_arg => {
        assert_equals(undefined_arg, undefined);
      });
  }, 'Importing a same-origin script with credentials=same-origin should ' +
     'include the credentials');

  promise_test(() => {
      const kSetCookieURL =
          get_host_info().HTTPS_REMOTE_ORIGIN +
          '/worklets/resources/set-cookie.py?name=cookieName';
      const kScriptURL = get_host_info().HTTPS_REMOTE_ORIGIN +
                         '/worklets/resources/credentials.py?mode=same-origin';
      const kOptions = { credentials: 'same-origin' };

      // Set a cookie in the remote origin and then start a worklet.
      return fetch(kSetCookieURL, { mode: 'cors' })
        .then(() => worklet.addModule(kScriptURL, kOptions))
        .then(undefined_arg => assert_equals(undefined_arg, undefined));
  }, 'Importing a remote-origin script with credentials=same-origin should ' +
     'not include the credentials');

  promise_test(() => {
      document.cookie = 'cookieName=include';
      const kScriptURL = 'resources/credentials.py?mode=include';
      const kOptions = { credentials: 'include' };
      return worklet.addModule(kScriptURL, kOptions).then(undefined_arg => {
          assert_equals(undefined_arg, undefined);
      });
  }, 'Importing a same-origin script with credentials=include should include ' +
     'the credentials');

  promise_test(() => {
      const kSetCookieURL =
          get_host_info().HTTPS_REMOTE_ORIGIN +
          '/worklets/resources/set-cookie.py?name=cookieName';
      const kScriptURL = get_host_info().HTTPS_REMOTE_ORIGIN +
                         '/worklets/resources/credentials.py?mode=include';
      const kOptions = { credentials: 'include' };

      // Set a cookie in the remote origin and then start a worklet.
      return fetch(kSetCookieURL, { mode: 'cors' })
        .then(() => worklet.addModule(kScriptURL, kOptions))
        .then(undefined_arg => assert_equals(undefined_arg, undefined));
  }, 'Importing a remote-origin script with credentials=include should ' +
     'include the credentials');
}
