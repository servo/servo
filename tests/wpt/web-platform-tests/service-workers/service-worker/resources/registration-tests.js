function registration_tests(register_method, check_error_types) {
  // The navigator.serviceWorker.register() method guarantees that the newly
  // installing worker is available as registration.installing when its promise
  // resolves. However these tests are also used to test installation using a
  // <link> element where it is possible for the installing worker to have
  // already become the waiting or active worker. So This method is used to get
  // the newest worker when these tests need access to the ServiceWorker itself.
  function get_newest_worker(registration) {
    if (registration.installing)
      return registration.installing;
    if (registration.waiting)
      return registration.waiting;
    if (registration.active)
      return registration.active;
  }

  promise_test(function(t) {
      var script = 'resources/registration-worker.js';
      var scope = 'resources/registration/normal';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_true(
              registration instanceof ServiceWorkerRegistration,
              'Successfully registered.');
            return registration.unregister();
          });
    }, 'Registering normal scope');

  promise_test(function(t) {
      var script = 'resources/registration-worker.js';
      var scope = 'resources/registration/scope-with-fragment#ref';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_true(
              registration instanceof ServiceWorkerRegistration,
              'Successfully registered.');
            assert_equals(
              registration.scope,
              normalizeURL('resources/registration/scope-with-fragment'),
              'A fragment should be removed from scope')
            return registration.unregister();
          });
    }, 'Registering scope with fragment');

  promise_test(function(t) {
      var script = 'resources/registration-worker.js';
      var scope = 'resources/';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_true(
              registration instanceof ServiceWorkerRegistration,
              'Successfully registered.');
            return registration.unregister();
          });
    }, 'Registering same scope as the script directory');

  promise_test(function(t) {
      var script = 'resources/registration-worker.js';
      var scope = 'resources';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registering same scope as the script directory without the last ' +
              'slash should fail with SecurityError.');
    }, 'Registering same scope as the script directory without the last slash');

  promise_test(function(t) {
      var script = 'resources/registration-worker.js';
      var scope = 'different-directory/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration scope outside the script directory should fail ' +
              'with SecurityError.');
    }, 'Registration scope outside the script directory');

  promise_test(function(t) {
      var script = 'resources/registration-worker.js';
      var scope = 'http://example.com/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration scope outside domain should fail with SecurityError.');
    }, 'Registering scope outside domain');

  promise_test(function(t) {
      var script = 'http://example.com/worker.js';
      var scope = 'http://example.com/scope/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration script outside domain should fail with SecurityError.');
    }, 'Registering script outside domain');

  promise_test(function(t) {
      var script = 'resources/no-such-worker.js';
      var scope = 'resources/scope/no-such-worker';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of non-existent script should fail.');
    }, 'Registering non-existent script');

  promise_test(function(t) {
      var script = 'resources/invalid-chunked-encoding.py';
      var scope = 'resources/scope/invalid-chunked-encoding/';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of invalid chunked encoding script should fail.');
    }, 'Registering invalid chunked encoding script');

  promise_test(function(t) {
      var script = 'resources/invalid-chunked-encoding-with-flush.py';
      var scope = 'resources/scope/invalid-chunked-encoding-with-flush/';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of invalid chunked encoding script should fail.');
    }, 'Registering invalid chunked encoding script with flush');

  promise_test(function(t) {
      var script = 'resources/mime-type-worker.py';
      var scope = 'resources/scope/no-mime-type-worker/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration of no MIME type script should fail.');
    }, 'Registering script with no MIME type');

  promise_test(function(t) {
      var script = 'resources/mime-type-worker.py?mime=text/plain';
      var scope = 'resources/scope/bad-mime-type-worker/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration of plain text script should fail.');
    }, 'Registering script with bad MIME type');

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

  promise_test(function(t) {
      var script = 'resources/import-mime-type-worker.py';
      var scope = 'resources/scope/no-mime-type-worker/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration of no MIME type imported script should fail.');
    }, 'Registering script that imports script with no MIME type');

  promise_test(function(t) {
      var script = 'resources/import-mime-type-worker.py?mime=text/plain';
      var scope = 'resources/scope/bad-mime-type-worker/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration of plain text imported script should fail.');
    }, 'Registering script that imports script with bad MIME type');


  promise_test(function(t) {
      var script = 'resources/redirect.py?Redirect=' +
                    encodeURIComponent('/resources/registration-worker.js');
      var scope = 'resources/scope/redirect/';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registration of redirected script should fail.');
    }, 'Registering redirected script');

  promise_test(function(t) {
      var script = 'resources/malformed-worker.py?parse-error';
      var scope = 'resources/scope/parse-error';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of script including parse error should fail.');
    }, 'Registering script including parse error');

  promise_test(function(t) {
      var script = 'resources/malformed-worker.py?undefined-error';
      var scope = 'resources/scope/undefined-error';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of script including undefined error should fail.');
    }, 'Registering script including undefined error');

  promise_test(function(t) {
      var script = 'resources/malformed-worker.py?uncaught-exception';
      var scope = 'resources/scope/uncaught-exception';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of script including uncaught exception should fail.');
    }, 'Registering script including uncaught exception');

  promise_test(function(t) {
      var script = 'resources/malformed-worker.py?caught-exception';
      var scope = 'resources/scope/caught-exception';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_true(
              registration instanceof ServiceWorkerRegistration,
              'Successfully registered.');
            return registration.unregister();
          });
    }, 'Registering script including caught exception');

  promise_test(function(t) {
      var script = 'resources/malformed-worker.py?import-malformed-script';
      var scope = 'resources/scope/import-malformed-script';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of script importing malformed script should fail.');
    }, 'Registering script importing malformed script');

  promise_test(function(t) {
      var script = 'resources/malformed-worker.py?import-no-such-script';
      var scope = 'resources/scope/import-no-such-script';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'Registration of script importing non-existent script should fail.');
    }, 'Registering script importing non-existent script');

  promise_test(function(t) {
      // URL-encoded full-width 'scope'.
      var name = '%ef%bd%93%ef%bd%83%ef%bd%8f%ef%bd%90%ef%bd%85';
      var script = 'resources/empty-worker.js';
      var scope = 'resources/' + name + '/escaped-multibyte-character-scope';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_equals(
              registration.scope,
              normalizeURL(scope),
              'URL-encoded multibyte characters should be available.');
            return registration.unregister();
          });
    }, 'Scope including URL-encoded multibyte characters');

  promise_test(function(t) {
      // Non-URL-encoded full-width "scope".
      var name = String.fromCodePoint(0xff53, 0xff43, 0xff4f, 0xff50, 0xff45);
      var script = 'resources/empty-worker.js';
      var scope = 'resources/' + name  + '/non-escaped-multibyte-character-scope';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_equals(
              registration.scope,
              normalizeURL(scope),
              'Non-URL-encoded multibyte characters should be available.');
            return registration.unregister();
          });
    }, 'Scope including non-escaped multibyte characters');

  promise_test(function(t) {
      var script = 'resources%2fempty-worker.js';
      var scope = 'resources/scope/encoded-slash-in-script-url';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'URL-encoded slash in the script URL should be rejected.');
    }, 'Script URL including URL-encoded slash');

  promise_test(function(t) {
      var script = 'resources%2Fempty-worker.js';
      var scope = 'resources/scope/encoded-slash-in-script-url';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'URL-encoded slash in the script URL should be rejected.');
    }, 'Script URL including uppercase URL-encoded slash');

  promise_test(function(t) {
      var script = 'resources/empty-worker.js';
      var scope = 'resources/scope%2fencoded-slash-in-scope';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'URL-encoded slash in the scope should be rejected.');
    }, 'Scope including URL-encoded slash');

  promise_test(function(t) {
      var script = 'resources%5cempty-worker.js';
      var scope = 'resources/scope/encoded-slash-in-script-url';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'URL-encoded backslash in the script URL should be rejected.');
    }, 'Script URL including URL-encoded backslash');

  promise_test(function(t) {
      var script = 'resources%5Cempty-worker.js';
      var scope = 'resources/scope/encoded-slash-in-script-url';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'URL-encoded backslash in the script URL should be rejected.');
    }, 'Script URL including uppercase URL-encoded backslash');

  promise_test(function(t) {
      var script = 'resources/empty-worker.js';
      var scope = 'resources/scope%5cencoded-slash-in-scope';
      return promise_rejects(t,
          check_error_types ? new TypeError : null,
          register_method(script, {scope: scope}),
          'URL-encoded backslash in the scope should be rejected.');
    }, 'Scope including URL-encoded backslash');

  promise_test(function(t) {
      var script = 'resources/././empty-worker.js';
      var scope = 'resources/scope/parent-reference-in-script-url';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_equals(
              get_newest_worker(registration).scriptURL,
              normalizeURL('resources/empty-worker.js'),
              'Script URL including self-reference should be normalized.');
            return registration.unregister();
          });
    }, 'Script URL including self-reference');

  promise_test(function(t) {
      var script = 'resources/empty-worker.js';
      var scope = 'resources/././scope/self-reference-in-scope';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_equals(
              registration.scope,
              normalizeURL('resources/scope/self-reference-in-scope'),
              'Scope including self-reference should be normalized.');
            return registration.unregister();
          });
    }, 'Scope including self-reference');

  promise_test(function(t) {
      var script = 'resources/../resources/empty-worker.js';
      var scope = 'resources/scope/parent-reference-in-script-url';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_equals(
              get_newest_worker(registration).scriptURL,
              normalizeURL('resources/empty-worker.js'),
              'Script URL including parent-reference should be normalized.');
            return registration.unregister();
          });
    }, 'Script URL including parent-reference');

  promise_test(function(t) {
      var script = 'resources/empty-worker.js';
      var scope = 'resources/../resources/scope/parent-reference-in-scope';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            assert_equals(
              registration.scope,
              normalizeURL('resources/scope/parent-reference-in-scope'),
              'Scope including parent-reference should be normalized.');
            return registration.unregister();
          });
    }, 'Scope including parent-reference');

  promise_test(function(t) {
      var script = 'resources/empty-worker.js';
      var scope = 'resources/../scope/parent-reference-in-scope';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Scope not under the script directory should be rejected.');
    }, 'Scope including parent-reference and not under the script directory');

  promise_test(function(t) {
      var script = 'resources////empty-worker.js';
      var scope = 'resources/scope/consecutive-slashes-in-script-url';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Consecutive slashes in the script url should not be unified.');
    }, 'Script URL including consecutive slashes');

  promise_test(function(t) {
      var script = 'resources/empty-worker.js';
      var scope = 'resources/scope////consecutive-slashes-in-scope';
      return register_method(script, {scope: scope})
        .then(function(registration) {
            // Although consecutive slashes in the scope are not unified, the
            // scope is under the script directory and registration should
            // succeed.
            assert_equals(
              registration.scope,
              normalizeURL(scope),
              'Should successfully be registered.');
            return registration.unregister();
          })
    }, 'Scope including consecutive slashes');

  promise_test(function(t) {
      var script = 'filesystem:' + normalizeURL('resources/empty-worker.js');
      var scope = 'resources/scope/filesystem-script-url';
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registering a script which has same-origin filesystem: URL should ' +
              'fail with SecurityError.');
    }, 'Script URL is same-origin filesystem: URL');

  promise_test(function(t) {
      var script = 'resources/empty-worker.js';
      var scope = 'filesystem:' + normalizeURL('resources/scope/filesystem-scope-url');
      return promise_rejects(t,
          check_error_types ? 'SecurityError' : null,
          register_method(script, {scope: scope}),
          'Registering with the scope that has same-origin filesystem: URL ' +
              'should fail with SecurityError.');
    }, 'Scope URL is same-origin filesystem: URL');
}
