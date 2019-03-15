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

// This unregisters the registration that precisely matches scope. Use this
// when unregistering by scope. If no registration is found, it just resolves.
function service_worker_unregister(test, scope) {
  var absoluteScope = (new URL(scope, window.location).href);
  return navigator.serviceWorker.getRegistration(scope)
    .then(function(registration) {
        if (registration && registration.scope === absoluteScope)
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

/**
 * Adds an iframe to the document and returns a promise that resolves to the
 * iframe when it finishes loading. The caller is responsible for removing the
 * iframe later if needed.
 *
 * @param {string} url
 * @returns {HTMLIFrameElement}
 */
function with_iframe(url) {
  return new Promise(function(resolve) {
      var frame = document.createElement('iframe');
      frame.className = 'test-iframe';
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

function base_path() {
  return location.pathname.replace(/\/[^\/]*$/, '/');
}

function test_login(test, origin, username, password, cookie) {
  return new Promise(function(resolve, reject) {
      with_iframe(
        origin + base_path() +
        'resources/fetch-access-control-login.html')
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

function test_websocket(test, frame, url) {
  return new Promise(function(resolve, reject) {
      var ws = new frame.contentWindow.WebSocket(url, ['echo', 'chat']);
      var openCalled = false;
      ws.addEventListener('open', test.step_func(function(e) {
          assert_equals(ws.readyState, 1, "The WebSocket should be open");
          openCalled = true;
          ws.close();
        }), true);

      ws.addEventListener('close', test.step_func(function(e) {
          assert_true(openCalled, "The WebSocket should be closed after being opened");
          resolve();
        }), true);

      ws.addEventListener('error', reject);
    });
}

function login_https(test) {
  var host_info = get_host_info();
  return test_login(test, host_info.HTTPS_REMOTE_ORIGIN,
                    'username1s', 'password1s', 'cookie1')
    .then(function() {
        return test_login(test, host_info.HTTPS_ORIGIN,
                          'username2s', 'password2s', 'cookie2');
      });
}

function websocket(test, frame) {
  return test_websocket(test, frame, get_websocket_url());
}

function get_websocket_url() {
  return 'wss://{{host}}:{{ports[wss][0]}}/echo';
}

// The navigator.serviceWorker.register() method guarantees that the newly
// installing worker is available as registration.installing when its promise
// resolves. However some tests test installation using a <link> element where
// it is possible for the installing worker to have already become the waiting
// or active worker. So this method is used to get the newest worker when these
// tests need access to the ServiceWorker itself.
function get_newest_worker(registration) {
  if (registration.installing)
    return registration.installing;
  if (registration.waiting)
    return registration.waiting;
  if (registration.active)
    return registration.active;
}

function register_using_link(script, options) {
  var scope = options.scope;
  var link = document.createElement('link');
  link.setAttribute('rel', 'serviceworker');
  link.setAttribute('href', script);
  link.setAttribute('scope', scope);
  document.getElementsByTagName('head')[0].appendChild(link);
  return new Promise(function(resolve, reject) {
        link.onload = resolve;
        link.onerror = reject;
      })
    .then(() => navigator.serviceWorker.getRegistration(scope));
}

function with_sandboxed_iframe(url, sandbox) {
  return new Promise(function(resolve) {
      var frame = document.createElement('iframe');
      frame.sandbox = sandbox;
      frame.src = url;
      frame.onload = function() { resolve(frame); };
      document.body.appendChild(frame);
    });
}

// Registers, waits for activation, then unregisters on a dummy scope.
//
// This can be used to wait for a period of time needed to register,
// activate, and then unregister a service worker.  When checking that
// certain behavior does *NOT* happen, this is preferable to using an
// arbitrary delay.
async function wait_for_activation_on_dummy_scope(t, window_or_workerglobalscope) {
  const script = '/service-workers/service-worker/resources/empty-worker.js';
  const scope = 'resources/there/is/no/there/there?' + Date.now();
  let registration = await window_or_workerglobalscope.navigator.serviceWorker.register(script, { scope });
  await wait_for_state(t, registration.installing, 'activated');
  await registration.unregister();
}

// This installs resources/appcache-ordering.manifest.
function install_appcache_ordering_manifest() {
  let resolve_install_appcache;
  let reject_install_appcache;

  // This is notified by the child iframe, i.e. appcache-ordering.install.html,
  // that's to be created below.
  window.notify_appcache_installed = success => {
    if (success)
      resolve_install_appcache();
    else
      reject_install_appcache();
  };

  return new Promise((resolve, reject) => {
      const frame = document.createElement('iframe');
      frame.src = 'resources/appcache-ordering.install.html';
      document.body.appendChild(frame);
      resolve_install_appcache = function() {
          document.body.removeChild(frame);
          resolve();
        };
      reject_install_appcache = function() {
          document.body.removeChild(frame);
          reject();
        };
  });
}

