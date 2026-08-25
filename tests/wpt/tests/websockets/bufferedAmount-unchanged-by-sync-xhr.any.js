// META: script=constants.sub.js
// META: global=window,dedicatedworker,sharedworker
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

async_test(t => {
  const ws = CreateWebSocket(false, false);
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
