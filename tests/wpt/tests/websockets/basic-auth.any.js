// META: global=window,worker
// META: script=constants.sub.js
// META: variant=?wss
// META: variant=?wpt_flags=h2

async_test(t => {
  const url = __SCHEME + '://' + 'foo:bar@' + __SERVER__NAME + ':' + __PORT + '/basic_auth';
  const ws = new WebSocket(url);
  ws.onopen = () => {
    ws.onclose = ws.onerror = null;
    ws.close();
    t.done();
  };
  ws.onerror = ws.onclose = t.unreached_func('open should succeed');
}, 'HTTP basic authentication should work with WebSockets');

done();
