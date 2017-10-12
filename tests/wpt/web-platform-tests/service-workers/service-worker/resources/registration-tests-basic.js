// Basic registration tests that succeed. We don't want too many successful
// registration tests in the same file since starting a service worker can be
// slow.
function registration_tests_basic(register_method, check_error_types) {
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
}
