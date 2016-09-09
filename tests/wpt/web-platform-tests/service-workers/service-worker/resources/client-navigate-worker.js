importScripts("worker-testharness.js");
importScripts("test-helpers.sub.js");
importScripts("get-host-info.sub.js")
importScripts("testharness-helpers.js")

self.onfetch = function(e) {
  if (e.request.url.indexOf("client-navigate-frame.html") >= 0) {
    if (e.clientId === null) {
      e.respondWith(fetch(e.request));
    } else {
      e.respondWith(Response.error());
    }
    return;
  }
  e.respondWith(new Response(e.clientId));
};

function pass(test, url) {
  return { result: test,
           url: url };
}

function fail(test, reason) {
  return { result: "FAILED " + test + " " + reason }
}

self.onmessage = function(e) {
  var port = e.data.port;
  var test = e.data.test;
  var clientId = e.data.clientId;
  var clientUrl = "";
  if (test === "test_client_navigate_success") {
    promise_test(function(t) {
      this.add_cleanup(() => port.postMessage(pass(test, clientUrl)));
      return self.clients.get(clientId)
                 .then(client => client.navigate("client-navigated-frame.html"))
                 .then(client => {
                   clientUrl = client.url;
                   assert_true(client instanceof WindowClient);
                 })
                 .catch(unreached_rejection(t));
    }, "Return value should be instance of WindowClient");
  } else if (test === "test_client_navigate_failure") {
    promise_test(function(t) {
      return self.clients.get(clientId)
                 .then(client => assert_promise_rejects(client.navigate("http://example.com")))
                 .catch(unreached_rejection(t));
    }, "Navigating to different origin should reject");

    promise_test(function(t) {
      this.add_cleanup(function() { port.postMessage(pass(test, "")); });
      return self.clients.get(clientId)
                 .then(client => promise_rejects(t, new TypeError(), client.navigate("about:blank")))
                 .catch(unreached_rejection(t));
    }, "Navigating to about:blank should reject with TypeError")
  } else if (test === "test_client_navigate_redirect") {
    var host_info = get_host_info();
    var url = new URL(host_info['HTTPS_REMOTE_ORIGIN']).toString() +
              new URL("client-navigated-frame.html", location).pathname.substring(1);
    promise_test(function(t) {
      this.add_cleanup(() => port.postMessage(pass(test, clientUrl)));
      return self.clients.get(clientId)
                 .then(client => client.navigate("redirect.py?Redirect=" + url))
                 .then(client => {
                   clientUrl = (client && client.url) || ""
                   assert_true(client === null);
                 })
                 .catch(unreached_rejection(t));
    }, "Redirecting to another origin should resolve with null");
  }
};
