// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

// A script acquiring a lock. It can be released using window.releaseLocks
const acquire_script = (key, response) =>  `
  window.releaseLocks ||= [];
  navigator.locks.request("${key}", async lock => {
    send("${response}", "locked")
    await new Promise(r => releaseLocks.push(r));
    send("${response}", "unlocked");
  });
`;

const release_script = (response) => `
  for (release of releaseLocks)
    release();
`;

// Assert that |context| holds |expected_keys|.
const assertHeldKeys = async (context, expected_keys) => {
  const queue = token();
  send(context, `
    const list = await navigator.locks.query();
    send("${queue}", JSON.stringify(list));
  `);
  const state = JSON.parse(await receive(queue));
  const held = state.held.map(x => x.name);
  assert_equals(held.length, expected_keys.length);
  assert_array_equals(held.sort(), expected_keys.sort());
}

promise_test(async test => {
  const origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const key_1 = token();
  const key_2 = token();

  // 2 actors: An anonymous iframe and a normal one.
  const iframe_anonymous = newAnonymousIframe(origin);
  const iframe_normal = newIframe(origin);
  const response_queue_1 = token();
  const response_queue_2 = token();

  // 1. Hold two different locks on both sides.
  send(iframe_anonymous, acquire_script(key_1, response_queue_1));
  send(iframe_normal, acquire_script(key_2, response_queue_2));
  assert_equals(await receive(response_queue_1), "locked");
  assert_equals(await receive(response_queue_2), "locked");
  await assertHeldKeys(iframe_anonymous, [key_1]);
  await assertHeldKeys(iframe_normal, [key_2]);

  // 2. Try to acquire the lock with the same key on the opposite side. It
  //    shouldn't block, because they are partitioned.
  send(iframe_anonymous , acquire_script(key_2, response_queue_1));
  send(iframe_normal, acquire_script(key_1, response_queue_2));
  assert_equals(await receive(response_queue_1), "locked");
  assert_equals(await receive(response_queue_2), "locked");
  await assertHeldKeys(iframe_anonymous, [key_1, key_2]);
  await assertHeldKeys(iframe_normal, [key_1, key_2]);

  // 3. Cleanup: release the 4 locks (2 on each sides).
  send(iframe_anonymous, release_script(response_queue_1));
  assert_equals(await receive(response_queue_1), "unlocked");
  assert_equals(await receive(response_queue_1), "unlocked");
  await assertHeldKeys(iframe_anonymous, []);
  await assertHeldKeys(iframe_normal, [key_1, key_2]);

  send(iframe_normal, release_script(response_queue_2));
  assert_equals(await receive(response_queue_2), "unlocked");
  assert_equals(await receive(response_queue_2), "unlocked");
  await assertHeldKeys(iframe_anonymous, []);
  await assertHeldKeys(iframe_normal, []);
})
