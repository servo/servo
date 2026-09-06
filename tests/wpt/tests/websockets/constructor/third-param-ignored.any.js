// META: title=WebSockets: third parameter ignored
// META: script=../constants.sub.js
// META: global=window,worker
// META: variant=?default
// META: variant=?wss

// Regression test for https://crbug.com/552159676.
promise_test(async (t) => {
  // The NodeJS "ws" library adds a third option bag parameter to the WebSocket
  // constructor. Some pages pass this even in the browser, for convenience when
  // sharing the same code between the browser and server, or for tests. Prior
  // to the addition of option bag support to the WebSocket constructor, passing
  // an `undefined` value as the second parameter results in requesting a
  // protocol called "undefined". The `echo_wsh.py` handler doesn't support
  // protocols other than "echo", so use `protocol_array_wsh.py` instead.
  const ws = new WebSocket(
      `${SCHEME_DOMAIN_PORT}/protocol_array`,
      undefined,
      {},
  );
  t.add_cleanup(() => ws.close());
  await new Promise((resolve) => {
    ws.onopen = t.step_func(() => ws.close());
    ws.onerror = t.unreached_func('error event should not have fired');
    ws.onclose = resolve;
  });
}, 'a third parameter to the constructor should be ignored');

promise_test(async (t) => {
  const ws = new WebSocket(
      `${SCHEME_DOMAIN_PORT}/echo`,
      'echo',
      {},
  );
  t.add_cleanup(() => ws.close());
  await new Promise(resolve => {
    ws.onopen = t.step_func(() => {
      assert_equals(ws.protocol, 'echo', 'protocol should be "echo"');
      ws.close();
    });
    ws.onerror = t.unreached_func('error event should not have fired');
    ws.onclose = resolve;
  });
}, 'protocol string should work with ignored third parameter');

promise_test(async (t) => {
  const ws = new WebSocket(
      `${SCHEME_DOMAIN_PORT}/echo`,
      ['echo'],
      {},
  );
  t.add_cleanup(() => ws.close());
  await new Promise(resolve => {
    ws.onopen = t.step_func(() => {
      assert_equals(ws.protocol, 'echo', 'protocol should be "echo"');
      ws.close();
    });
    ws.onerror = t.unreached_func('error event should not have fired');
    ws.onclose = resolve;
  });
}, 'protocol sequence should work with ignored third parameter');
