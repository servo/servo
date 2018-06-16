// META: script=websocket.sub.js
// META: global=sharedworker,serviceworker

async_test(t => {
  const isSecure = new URL(location.href).scheme === 'https';
  const scheme = isSecure ? 'wss' : 'ws';
  const port = isSecure ? __SECURE__PORT : __PORT;
  const url = scheme + '://' + 'foo:bar@' + __SERVER__NAME + ':' + port + '/basic_auth';
  const ws = new WebSocket(url);
  ws.onopen = () => {
    ws.onclose = ws.onerror = null;
    ws.close();
    t.done();
  };
  ws.onerror = ws.onclose = t.unreached_func('open should succeed');
}, 'HTTP basic authentication should work with WebSockets');

done();
