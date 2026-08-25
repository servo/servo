// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

// "token()" is used to get unique value for every execution of the test. This
// avoids potential side effects of one run toward the second.
const g_db_store = token();
const g_db_name = token();
const g_db_version = 1;

// A script storing "|id|=|value|" in IndexedDB.
const write_script = (id, value, done) => `
  // Open the database:
  const request = indexedDB.open("${g_db_name}", "${g_db_version}");
  request.onupgradeneeded = () => {
    request.result.createObjectStore("${g_db_store}", {keyPath: "id"});
  };
  await new Promise(r => request.onsuccess = r);
  const db = request.result;

  // Write the value:
  const transaction_write = db.transaction("${g_db_store}", "readwrite");
  transaction_write.objectStore("${g_db_store}").add({
    id: "${id}",
    value: "${value}",
  });
  await transaction_write.complete;

  db.close();
  send("${done}", "Done");
`;

// A script retrieving what was stored inside IndexedDB.
const read_script = (done) => `
  // Open the database:
  const request = indexedDB.open("${g_db_name}", "${g_db_version}");
  await new Promise(r => request.onsuccess = r);
  const db = request.result;

  // Read:
  const transaction_read = db.transaction("${g_db_store}", "readonly");
  const get_all = transaction_read.objectStore("${g_db_store}").getAll();
  await new Promise(r => transaction_read.oncomplete = r);

  db.close();
  send("${done}", JSON.stringify(get_all.result));
`;

promise_test(async test => {
  // 4 actors: 2 credentialless iframe and 2 normal iframe.
  const origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const iframes = [
    newIframeCredentialless(origin),
    newIframeCredentialless(origin),
    newIframe(origin),
    newIframe(origin),
  ];

  // 1. Write a different key-value pair from the iframes in IndexedDB:
  const keys = iframes.map(token);
  const values = iframes.map(token);
  const response_queues = iframes.map(token);
  await Promise.all(iframes.map(async (_, i) => {
    send(iframes[i], write_script(keys[i], values[i], response_queues[i]));
    assert_equals(await receive(response_queues[i]), "Done");
  }));

  // 2. Read the state from every iframes:
  const states = await Promise.all(iframes.map(async (_, i) => {
    send(iframes[i], read_script(response_queues[i]));
    const reply = JSON.parse(await receive(response_queues[i]));

    const state = {}
    for(entry of reply)
      state[entry.id] = entry.value;
    return state;
  }));


  // Verify the two credentialless iframe share the same state and the normal
  // iframe share a second state
  assert_equals(states[0][keys[0]], values[0]);
  assert_equals(states[0][keys[1]], values[1]);
  assert_equals(states[0][keys[2]], undefined);
  assert_equals(states[0][keys[3]], undefined);

  assert_equals(states[1][keys[0]], values[0]);
  assert_equals(states[1][keys[1]], values[1]);
  assert_equals(states[1][keys[2]], undefined);
  assert_equals(states[1][keys[3]], undefined);

  assert_equals(states[2][keys[0]], undefined);
  assert_equals(states[2][keys[1]], undefined);
  assert_equals(states[2][keys[2]], values[2]);
  assert_equals(states[2][keys[3]], values[3]);

  assert_equals(states[3][keys[0]], undefined);
  assert_equals(states[3][keys[1]], undefined);
  assert_equals(states[3][keys[2]], values[2]);
  assert_equals(states[3][keys[3]], values[3]);
})
