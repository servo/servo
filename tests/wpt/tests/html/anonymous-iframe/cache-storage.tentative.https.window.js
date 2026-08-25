// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

// A script storing a value into the CacheStorage.
const store_script = (key, value, done) =>  `
  const request = new Request("/${key}.txt");
  const response = new Response("${value}", {
    headers: { "content-type": "plain/txt" }
  });
  const cache = await caches.open("v1");
  const value = await cache.put(request, response.clone());
  send("${done}", "stored");
`;

// A script loading a value from the CacheStorage.
const load_script = (key, done) => `
  const cache = await caches.open("v1");
  const request = new Request("/${key}.txt");
  try {
    const response = await cache.match(request);
    const value = await response.text();
    send("${done}", value);
  } catch (error) {
    send("${done}", "not found");
  }
`;

promise_test(async test => {
  const origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const key_1 = token();
  const key_2 = token();

  // 2 actors: A credentialless iframe and a normal one.
  const iframe_credentialless = newIframeCredentialless(origin);
  const iframe_normal = newIframe(origin);
  const response_queue_1 = token();
  const response_queue_2 = token();

  // 1. Each of them store a value in CacheStorage with different keys.
  send(iframe_credentialless , store_script(key_1, "value_1", response_queue_1));
  send(iframe_normal, store_script(key_2, "value_2", response_queue_2));
  assert_equals(await receive(response_queue_1), "stored");
  assert_equals(await receive(response_queue_2), "stored");

  // 2. Each of them tries to retrieve the value from opposite side, without
  //    success.
  send(iframe_credentialless , load_script(key_2, response_queue_1));
  send(iframe_normal, load_script(key_1, response_queue_2));
  assert_equals(await receive(response_queue_1), "not found");
  assert_equals(await receive(response_queue_2), "not found");

  // 3. Each of them tries to retrieve the value from their side, with success:
  send(iframe_credentialless , load_script(key_1, response_queue_1));
  send(iframe_normal, load_script(key_2, response_queue_2));
  assert_equals(await receive(response_queue_1), "value_1");
  assert_equals(await receive(response_queue_2), "value_2");
})
