// META: title=WebSockets: targetAddressSpace option in constructor argument
// META: script=../constants.sub.js
// META: variant=?default
// META: variant=?wss

async_test(t => {
  const ws = new WebSocket(SCHEME_DOMAIN_PORT + '/echo',
                           {targetAddressSpace: 'loopback'});
  ws.onopen = t.step_func(() => {
    ws.close();
    t.done();
  });
  ws.onerror = t.unreached_func('error event should not have fired');
}, 'WebSocket constructor with matching targetAddressSpace \'loopback\' succeeds');

test(() => {
  assert_throws_js(TypeError, () => {
    new WebSocket(SCHEME_DOMAIN_PORT + '/echo',
                  {targetAddressSpace: 'private'});
  });
}, 'WebSocket constructor with targetAddressSpace \'private\' throws TypeError as it is an unsupported legacy alias');

async_test(t => {
  const ws = new WebSocket(SCHEME_DOMAIN_PORT + '/echo',
                           {targetAddressSpace: 'local'});
  ws.onopen = t.unreached_func('open event should not have fired');
  ws.onerror = t.step_func(() => {
    t.done();
  });
}, 'WebSocket constructor with mismatched targetAddressSpace \'local\' fails connection');

async_test(t => {
  const ws = new WebSocket(SCHEME_DOMAIN_PORT + '/echo',
                           {targetAddressSpace: 'public'});
  ws.onopen = t.unreached_func('open event should not have fired');
  ws.onerror = t.step_func(() => {
    t.done();
  });
}, 'WebSocket constructor with mismatched targetAddressSpace \'public\' fails connection');

test(() => {
  assert_throws_js(TypeError, () => {
    new WebSocket(SCHEME_DOMAIN_PORT + '/echo',
                  {targetAddressSpace: 'unknown'});
  });
}, 'WebSocket constructor with targetAddressSpace \'unknown\' throws TypeError');

test(() => {
  assert_throws_js(TypeError, () => {
    new WebSocket(SCHEME_DOMAIN_PORT + '/echo',
                  {targetAddressSpace: 'invalid'});
  });
}, 'WebSocket constructor with invalid targetAddressSpace throws TypeError');
