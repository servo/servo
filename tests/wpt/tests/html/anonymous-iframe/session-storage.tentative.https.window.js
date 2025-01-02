// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

// Make |iframe| to store |key|=|value| into sessionStorage.
const store = async (iframe, key, value) => {
  const response_queue = token();
  send(iframe, `
    sessionStorage.setItem("${key}", "${value}");
    send("${response_queue}", "stored");
  `);
  assert_equals(await receive(response_queue), "stored");
};

// Make |iframe| to load |key| in sessionStorage. Check it matches the
// |expected_value|.
const load = async (iframe, key, expected_value) => {
  const response_queue = token();
  send(iframe, `
    const value = sessionStorage.getItem("${key}");
    send("${response_queue}", value || "not found");
  `);
  assert_equals(await receive(response_queue), expected_value);
};

promise_test(async test => {
  const origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const key_1 = token();
  const key_2 = token();

  // 4 actors: 2 credentialless iframe and 2 normal iframe.
  const iframe_credentialless_1 = newIframeCredentialless(origin);
  const iframe_credentialless_2 = newIframeCredentialless(origin);
  const iframe_normal_1 = newIframe(origin);
  const iframe_normal_2 = newIframe(origin);

  // 1. Store a value in one credentialless iframe and one normal iframe.
  await Promise.all([
    store(iframe_credentialless_1, key_1, "value_1"),
    store(iframe_normal_1, key_2, "value_2"),
  ]);

  // 2. Check what each of them can retrieve.
  await Promise.all([
    load(iframe_credentialless_1, key_1, "value_1"),
    load(iframe_credentialless_2, key_1, "value_1"),
    load(iframe_credentialless_1, key_2, "not found"),
    load(iframe_credentialless_2, key_2, "not found"),

    load(iframe_normal_1, key_1, "not found"),
    load(iframe_normal_2, key_1, "not found"),
    load(iframe_normal_1, key_2, "value_2"),
    load(iframe_normal_2, key_2, "value_2"),
  ]);
}, "Session storage is correctly partitioned with regards to credentialless iframe");
