// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=../credentialless/resources/common.js
// META: script=./resources/common.js

// A script listening using a BroadcastChannel.
const listen_script = (key, done, onmessage) => `
  const bc = new BroadcastChannel("${key}");
  bc.onmessage = event => send("${onmessage}", event.data);
  send("${done}", "registered");
`;

const emit_script = (key, message) => `
  const bc = new BroadcastChannel("${key}");
  bc.postMessage("${message}");
`;

promise_test(async test => {
  const origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const key_1 = token();
  const key_2 = token();

  // 2 actors: An anonymous iframe and a normal one.
  const iframe_anonymous = newAnonymousIframe(origin);
  const iframe_normal = newIframe(origin);
  const queue_1 = token();
  const queue_2 = token();
  const unexpected_queue = token();

  // Listen using the two keys from both sides:
  send(iframe_anonymous , listen_script(key_1, queue_1, queue_1));
  send(iframe_anonymous , listen_script(key_2, queue_1, unexpected_queue));
  send(iframe_normal, listen_script(key_2, queue_2, queue_2));
  send(iframe_normal, listen_script(key_1, queue_2, unexpected_queue));
  assert_equals(await receive(queue_1), "registered");
  assert_equals(await receive(queue_1), "registered");
  assert_equals(await receive(queue_2), "registered");
  assert_equals(await receive(queue_2), "registered");

  // Emit from both sides. It must work, and work without crossing the
  // anonymous/non-anonymous border.
  receive(unexpected_queue).then(test.unreached_func(
    "BroadcastChannel shouldn't cross the anonymous/normal border"));
  send(iframe_anonymous , emit_script(key_1, "msg_1"));
  send(iframe_normal, emit_script(key_2, "msg_2"));
  assert_equals(await receive(queue_1), "msg_1");
  assert_equals(await receive(queue_2), "msg_2");

  // Wait a bit to let bad things the opportunity to show up. This is done by
  // repeating the previous operation.
  send(iframe_anonymous , emit_script(key_1, "msg_3"));
  send(iframe_normal, emit_script(key_2, "msg_4"));
  assert_equals(await receive(queue_1), "msg_3");
  assert_equals(await receive(queue_2), "msg_4");
})
