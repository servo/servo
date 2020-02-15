importScripts('/resources/testharness.js');

function match_query(query_string) {
  return self.location.search.substr(1) == query_string;
}

async function navigate_test(t, e) {
  var port = e.data.port;
  var url = e.data.url;
  var expected = e.data.expected;

  var p = clients.matchAll({ includeUncontrolled : true })
    .then(function(client_list) {
        for (var i = 0; i < client_list.length; i++) {
          var client = client_list[i];
          if (client.frameType == 'nested') {
            return client.navigate(url);
          }
        }
        throw 'Could not locate window client.';
      })
    .then(function(new_client) {
        // If we didn't reject, we better get resolved with the right thing.
        if (new_client === null) {
          assert_equals(new_client, expected);
        } else {
          assert_equals(new_client.url, expected);
        }
      });

  if (typeof self[expected] == "function") {
    // It's a JS error type name.  We are expecting our promise to be rejected
    // with that error.
    p = promise_rejects_js(t, self[expected], p);
  }

  // Let our caller know we are done.
  return p.finally(() => port.postMessage(null));
}

function getTestClient() {
  return clients.matchAll({ includeUncontrolled: true })
    .then(function(client_list) {
        for (var i = 0; i < client_list.length; i++) {
          var client = client_list[i];

          if (/windowclient-navigate\.https\.html/.test(client.url)) {
            return client;
          }
        }

        throw new Error('Service worker was unable to locate test client.');
      });
}

function waitForMessage(client) {
  var channel = new MessageChannel();
  client.postMessage({ port: channel.port2 }, [channel.port2]);

  return new Promise(function(resolve) {
        channel.port1.onmessage = resolve;
      });
}

// The worker must remain in the "installing" state for the duration of some
// sub-tests. In order to achieve this coordination without relying on global
// state, the worker must create a message channel with the client from within
// the "install" event handler.
if (match_query('installing')) {
  self.addEventListener('install', function(e) {
      e.waitUntil(getTestClient().then(waitForMessage));
    });
}

self.addEventListener('message', function(e) {
    e.waitUntil(promise_test(t => navigate_test(t, e),
                             e.data.description + " worker side"));
  });
