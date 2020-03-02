// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js

promise_test(async (test) => {
  const origin = get_host_info().REMOTE_ORIGIN;
  const id = token();
  const store = `${origin}/beacon/resources/beacon.py?cmd=store&id=${id}`;
  const monitor = `/beacon/resources/beacon.py?cmd=stat&id=${id}`;

  assert_true(navigator.sendBeacon(store, new Blob([], {type: 'x/y'})));

  let actual;
  for (let i = 0; i < 30; ++i) {
    await new Promise(resolve => test.step_timeout(resolve, 10));

    const response = await fetch(monitor);
    const obj = await response.json();
    if (obj.length > 0) {
      actual = JSON.stringify(obj);
      break;
    }
  }

  const expected =
    JSON.stringify([{error: 'Preflight not expected.'}]);

  assert_equals(actual, expected);
});
