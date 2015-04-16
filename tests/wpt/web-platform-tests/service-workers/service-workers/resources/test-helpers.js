// Adapter for testharness.js-style tests with Service Workers

function service_worker_unregister_and_register(test, url, scope) {
  if (!scope || scope.length == 0)
    return Promise.reject(new Error('tests must define a scope'));

  var options = { scope: scope };
  return service_worker_unregister(test, scope)
    .then(function() {
        return navigator.serviceWorker.register(url, options);
      })
    .catch(unreached_rejection(test,
                               'unregister and register should not fail'));
}

function service_worker_unregister(test, documentUrl) {
  return navigator.serviceWorker.getRegistration(documentUrl)
    .then(function(registration) {
        if (registration)
          return registration.unregister();
      })
    .catch(unreached_rejection(test, 'unregister should not fail'));
}

function service_worker_unregister_and_done(test, scope) {
  return service_worker_unregister(test, scope)
    .then(test.done.bind(test));
}

function unreached_fulfillment(test, prefix) {
  return test.step_func(function(result) {
      var error_prefix = prefix || 'unexpected fulfillment';
      assert_unreached(error_prefix + ': ' + result);
    });
}

// Rejection-specific helper that provides more details
function unreached_rejection(test, prefix) {
  return test.step_func(function(error) {
      var reason = error.message || error.name || error;
      var error_prefix = prefix || 'unexpected rejection';
      assert_unreached(error_prefix + ': ' + reason);
    });
}

// Adds an iframe to the document and returns a promise that resolves to the
// iframe when it finishes loading. The caller is responsible for removing the
// iframe later if needed.
function with_iframe(url) {
  return new Promise(function(resolve) {
      var frame = document.createElement('iframe');
      frame.src = url;
      frame.onload = function() { resolve(frame); };
      document.body.appendChild(frame);
    });
}

function normalizeURL(url) {
  return new URL(url, self.location).toString().replace(/#.*$/, '');
}

function wait_for_update(test, registration) {
  if (!registration || registration.unregister == undefined) {
    return Promise.reject(new Error(
      'wait_for_update must be passed a ServiceWorkerRegistration'));
  }

  return new Promise(test.step_func(function(resolve) {
      registration.addEventListener('updatefound', test.step_func(function() {
          resolve(registration.installing);
        }));
    }));
}

function wait_for_state(test, worker, state) {
  if (!worker || worker.state == undefined) {
    return Promise.reject(new Error(
      'wait_for_state must be passed a ServiceWorker'));
  }
  if (worker.state === state)
    return Promise.resolve(state);

  if (state === 'installing') {
    switch (worker.state) {
      case 'installed':
      case 'activating':
      case 'activated':
      case 'redundant':
        return Promise.reject(new Error(
          'worker is ' + worker.state + ' but waiting for ' + state));
    }
  }

  if (state === 'installed') {
    switch (worker.state) {
      case 'activating':
      case 'activated':
      case 'redundant':
        return Promise.reject(new Error(
          'worker is ' + worker.state + ' but waiting for ' + state));
    }
  }

  if (state === 'activating') {
    switch (worker.state) {
      case 'activated':
      case 'redundant':
        return Promise.reject(new Error(
          'worker is ' + worker.state + ' but waiting for ' + state));
    }
  }

  if (state === 'activated') {
    switch (worker.state) {
      case 'redundant':
        return Promise.reject(new Error(
          'worker is ' + worker.state + ' but waiting for ' + state));
    }
  }

  return new Promise(test.step_func(function(resolve) {
      worker.addEventListener('statechange', test.step_func(function() {
          if (worker.state === state)
            resolve(state);
        }));
    }));
}

// Declare a test that runs entirely in the ServiceWorkerGlobalScope. The |url|
// is the service worker script URL. This function:
// - Instantiates a new test with the description specified in |description|.
//   The test will succeed if the specified service worker can be successfully
//   registered and installed.
// - Creates a new ServiceWorker registration with a scope unique to the current
//   document URL. Note that this doesn't allow more than one
//   service_worker_test() to be run from the same document.
// - Waits for the new worker to begin installing.
// - Imports tests results from tests running inside the ServiceWorker.
function service_worker_test(url, description) {
  // If the document URL is https://example.com/document and the script URL is
  // https://example.com/script/worker.js, then the scope would be
  // https://example.com/script/scope/document.
  var scope = new URL('scope' + window.location.pathname,
                      new URL(url, window.location)).toString();
  promise_test(function(test) {
      return service_worker_unregister_and_register(test, url, scope)
        .then(function(registration) {
            add_completion_callback(function() {
                registration.unregister();
              });
            return wait_for_update(test, registration)
              .then(function(worker) {
                  return fetch_tests_from_worker(worker);
                });
          });
    }, description);
}

function get_host_info() {
  var ORIGINAL_HOST = '127.0.0.1';
  var REMOTE_HOST = 'localhost';
  var UNAUTHENTICATED_HOST = 'example.test';
  var HTTP_PORT = 8000;
  var HTTPS_PORT = 8443;
  try {
    // In W3C test, we can get the hostname and port number in config.json
    // using wptserve's built-in pipe.
    // http://wptserve.readthedocs.org/en/latest/pipes.html#built-in-pipes
    HTTP_PORT = eval('{{ports[http][0]}}');
    HTTPS_PORT = eval('{{ports[https][0]}}');
    ORIGINAL_HOST = eval('\'{{host}}\'');
    REMOTE_HOST = 'www1.' + ORIGINAL_HOST;
  } catch (e) {
  }
  return {
    HTTP_ORIGIN: 'http://' + ORIGINAL_HOST + ':' + HTTP_PORT,
    HTTPS_ORIGIN: 'https://' + ORIGINAL_HOST + ':' + HTTPS_PORT,
    HTTP_REMOTE_ORIGIN: 'http://' + REMOTE_HOST + ':' + HTTP_PORT,
    HTTPS_REMOTE_ORIGIN: 'https://' + REMOTE_HOST + ':' + HTTPS_PORT,
    UNAUTHENTICATED_ORIGIN: 'http://' + UNAUTHENTICATED_HOST + ':' + HTTP_PORT
  };
}

function base_path() {
  return location.pathname.replace(/\/[^\/]*$/, '/');
}

function test_login(test, origin, username, password, cookie) {
  return new Promise(function(resolve, reject) {
      with_iframe(
        origin +
        '/serviceworker/resources/fetch-access-control-login.html')
        .then(test.step_func(function(frame) {
            var channel = new MessageChannel();
            channel.port1.onmessage = test.step_func(function() {
                frame.remove();
                resolve();
              });
            frame.contentWindow.postMessage(
              {username: username, password: password, cookie: cookie},
              origin, [channel.port2]);
          }));
    });
}

function login(test) {
  return test_login(test, 'http://127.0.0.1:8000',
                    'username1', 'password1', 'cookie1')
    .then(function() {
        return test_login(test, 'http://localhost:8000',
                          'username2', 'password2', 'cookie2');
      });
}

function login_https(test) {
  return test_login(test, 'https://127.0.0.1:8443',
                    'username1s', 'password1s', 'cookie1')
    .then(function() {
        return test_login(test, 'https://localhost:8443',
                          'username2s', 'password2s', 'cookie2');
      });
}
