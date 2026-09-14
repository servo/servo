// META: global=window,worker
test(() => {
  const wt = new WebTransport('https://localhost:0/');
  wt.ready.catch(() => {});
  wt.closed.catch(() => {});
  assert_equals(wt.reliability, 'pending',
                'reliability is pending synchronously after construction');
  wt.close();
}, 'WebTransport reliability is pending before connection establishment');
