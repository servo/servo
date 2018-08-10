// META: script=websocket.sub.js
// META: global=sharedworker

async_test(t => {
  const url = 'wss://' + __SERVER__NAME + ':' + __SECURE__PORT + '/echo';
  const ws = new WebSocket(url);
  ws.onopen = t.step_func(() => {
    ws.onclose = ws.onerror = null;
    assert_equals(ws.bufferedAmount, 0);
    ws.send('hello');
    assert_equals(ws.bufferedAmount, 5);
    // Stop execution for 1s with a sync XHR.
    const xhr = new XMLHttpRequest();
    xhr.open('GET', '/common/blank.html?pipe=trickle(d1)', false);
    xhr.send();
    assert_equals(ws.bufferedAmount, 5);
    ws.close();
    t.done();
  });
  ws.onerror = ws.onclose = t.unreached_func('open should succeed');
}, 'bufferedAmount should not be updated during a sync XHR');

done();
