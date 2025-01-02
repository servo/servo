// Registration tests that mostly verify the MIME type.
//
// This file tests every MIME type so it necessarily starts many service
// workers, so it may be slow.
function registration_tests_mime_types(register_method) {
  promise_test(function(t) {
      var script = 'resources/mime-type-worker.py';
      var scope = 'resources/scope/no-mime-type-worker/';
      return promise_rejects_dom(t,
          'SecurityError',
          register_method(script, {scope: scope}),
          'Registration of no MIME type script should fail.');
    }, 'Registering script with no MIME type');

  promise_test(function(t) {
      var script = 'resources/mime-type-worker.py?mime=text/plain';
      var scope = 'resources/scope/bad-mime-type-worker/';
      return promise_rejects_dom(t,
          'SecurityError',
          register_method(script, {scope: scope}),
          'Registration of plain text script should fail.');
    }, 'Registering script with bad MIME type');

  /**
   * ServiceWorkerContainer.register() should throw a TypeError, according to
   * step 17.1 of https://w3c.github.io/ServiceWorker/#importscripts
   *
   * "[17] If an uncaught runtime script error occurs during the above step, then:
   *  [17.1] Invoke Reject Job Promise with job and TypeError"
   *
   * (Where the "uncaught runtime script error" is thrown by an unsuccessful
   * importScripts())
   */
  promise_test(function(t) {
      var script = 'resources/import-mime-type-worker.py';
      var scope = 'resources/scope/no-mime-type-worker/';
      return promise_rejects_js(t,
          TypeError,
          register_method(script, {scope: scope}),
          'Registration of no MIME type imported script should fail.');
    }, 'Registering script that imports script with no MIME type');

  promise_test(function(t) {
      var script = 'resources/import-mime-type-worker.py?mime=text/plain';
      var scope = 'resources/scope/bad-mime-type-worker/';
      return promise_rejects_js(t,
          TypeError,
          register_method(script, {scope: scope}),
          'Registration of plain text imported script should fail.');
    }, 'Registering script that imports script with bad MIME type');

  const validMimeTypes = [
    'application/ecmascript',
    'application/javascript',
    'application/x-ecmascript',
    'application/x-javascript',
    'text/ecmascript',
    'text/javascript',
    'text/javascript1.0',
    'text/javascript1.1',
    'text/javascript1.2',
    'text/javascript1.3',
    'text/javascript1.4',
    'text/javascript1.5',
    'text/jscript',
    'text/livescript',
    'text/x-ecmascript',
    'text/x-javascript'
  ];

  for (const validMimeType of validMimeTypes) {
    promise_test(() => {
      var script = `resources/mime-type-worker.py?mime=${validMimeType}`;
      var scope = 'resources/scope/good-mime-type-worker/';

      return register_method(script, {scope}).then(registration => {
        assert_true(
          registration instanceof ServiceWorkerRegistration,
          'Successfully registered.');
        return registration.unregister();
      });
    }, `Registering script with good MIME type ${validMimeType}`);

    promise_test(() => {
      var script = `resources/import-mime-type-worker.py?mime=${validMimeType}`;
      var scope = 'resources/scope/good-mime-type-worker/';

      return register_method(script, { scope }).then(registration => {
        assert_true(
          registration instanceof ServiceWorkerRegistration,
          'Successfully registered.');
        return registration.unregister();
      });
    }, `Registering script that imports script with good MIME type ${validMimeType}`);
  }
}
