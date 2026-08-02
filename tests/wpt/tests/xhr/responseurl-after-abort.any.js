// META: title=XMLHttpRequest: responseURL is the empty string after abort()

// https://xhr.spec.whatwg.org/#the-abort()-method
// abort() sets the response to a network error, both via the request error steps (step 2) and
// when the state is already done (step 3). A network error's URL is null, so responseURL must
// return the empty string, just like status is 0 and getAllResponseHeaders() is "".

const url = "resources/well-formed.xml";

function assertNetworkError(client) {
  assert_equals(client.status, 0, "status");
  assert_equals(client.statusText, "", "statusText");
  assert_equals(client.getAllResponseHeaders(), "", "getAllResponseHeaders()");
  assert_equals(client.responseURL, "", "responseURL");
}

async_test(test => {
  const client = new XMLHttpRequest();
  client.onreadystatechange = test.step_func(() => {
    if (client.readyState !== 2)
      return;
    assert_equals(client.status, 200);
    assert_not_equals(client.responseURL, "", "responseURL before abort()");
    client.abort();
    assertNetworkError(client);
  });
  client.onloadend = test.step_func_done(() => {
    assertNetworkError(client);
  });
  client.open("GET", url);
  client.send(null);
}, "abort() during HEADERS_RECEIVED clears responseURL");

async_test(test => {
  const client = new XMLHttpRequest();
  let aborted = false;
  client.onreadystatechange = test.step_func(() => {
    if (client.readyState !== 3 || aborted)
      return;
    aborted = true;
    assert_equals(client.status, 200);
    assert_not_equals(client.responseURL, "", "responseURL before abort()");
    client.abort();
    assertNetworkError(client);
  });
  client.onloadend = test.step_func_done(() => {
    assert_true(aborted, "reached LOADING");
    assertNetworkError(client);
  });
  client.open("GET", url);
  client.send(null);
}, "abort() during LOADING clears responseURL");

async_test(test => {
  const client = new XMLHttpRequest();
  client.onload = test.step_func_done(() => {
    assert_equals(client.readyState, 4);
    assert_not_equals(client.responseURL, "", "responseURL before abort()");
    client.abort();
    assert_equals(client.readyState, 0);
    assertNetworkError(client);
  });
  client.open("GET", url);
  client.send(null);
}, "abort() during DONE clears responseURL (async)");

test(() => {
  const client = new XMLHttpRequest();
  client.open("GET", url, false);
  client.send(null);
  assert_equals(client.readyState, 4);
  assert_not_equals(client.responseURL, "", "responseURL before abort()");
  client.abort();
  assert_equals(client.readyState, 0);
  assertNetworkError(client);
}, "abort() during DONE clears responseURL (sync)");

async_test(test => {
  const client = new XMLHttpRequest();
  client.open("GET", url);
  client.send(null);
  client.abort();
  assertNetworkError(client);
  // A reused XMLHttpRequest must report the new response's URL, not the aborted one.
  client.onload = test.step_func_done(() => {
    assert_equals(client.status, 200);
    assert_equals(client.responseURL, new URL(url + "?reused", location.href).href);
  });
  client.open("GET", url + "?reused");
  client.send(null);
}, "responseURL reflects the new response after abort() and reuse");
