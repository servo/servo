// META: global=window
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable();

  assert_true(writable instanceof WebTransportDatagramsWritable,
              'createWritable returns WebTransportDatagramsWritable');
  assert_true(writable instanceof WritableStream,
              'WebTransportDatagramsWritable extends WritableStream');
}, 'createWritable returns WebTransportDatagramsWritable which extends WritableStream');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable();

  assert_equals(writable.sendOrder, 0, 'sendOrder defaults to 0');
  assert_equals(writable.sendGroup, null, 'sendGroup defaults to null');
}, 'WebTransportDatagramsWritable attributes have correct defaults');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable({ sendOrder: 42 });

  assert_equals(writable.sendOrder, 42, 'sendOrder is set from options');
}, 'createWritable respects sendOrder option');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable();

  writable.sendOrder = 100;
  assert_equals(writable.sendOrder, 100, 'sendOrder can be set');

  writable.sendOrder = -50;
  assert_equals(writable.sendOrder, -50, 'sendOrder can be negative');

  writable.sendOrder = 0;
  assert_equals(writable.sendOrder, 0, 'sendOrder can be reset to 0');
}, 'sendOrder attribute can be get and set');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const sendGroup = wt.createSendGroup();
  const writable = wt.datagrams.createWritable();

  writable.sendGroup = sendGroup;
  assert_equals(writable.sendGroup, sendGroup, 'sendGroup can be set');

  writable.sendGroup = null;
  assert_equals(writable.sendGroup, null, 'sendGroup can be set to null');
}, 'sendGroup attribute can be get and set');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable();
  const writer = writable.getWriter();
  const reader = wt.datagrams.readable.getReader();

  const encoder = new TextEncoder();
  const data = encoder.encode('test message');

  writer.write(data).catch(() => {});

  const { value, done } = await reader.read();
  assert_false(done, 'read should not be done');

  const decoder = new TextDecoder();
  const received = decoder.decode(value);
  assert_equals(received, 'test message', 'datagram echoed correctly');
}, 'WebTransportDatagramsWritable can write and receive datagrams');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable1 = wt.datagrams.createWritable({ sendOrder: 10 });
  const writable2 = wt.datagrams.createWritable({ sendOrder: 20 });

  assert_not_equals(writable1, writable2, 'multiple writables are distinct');
  assert_equals(writable1.sendOrder, 10, 'writable1 has correct sendOrder');
  assert_equals(writable2.sendOrder, 20, 'writable2 has correct sendOrder');
}, 'multiple WebTransportDatagramsWritable streams can be created');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable({ sendOrder: 5 });
  const writer = writable.getWriter();
  const reader = wt.datagrams.readable.getReader();

  const encoder = new TextEncoder();

  writer.write(encoder.encode('message 1')).catch(() => {});
  writer.write(encoder.encode('message 2')).catch(() => {});
  writer.write(encoder.encode('message 3')).catch(() => {});

  const messages = [];
  for (let i = 0; i < 3; i++) {
    const { value, done } = await reader.read();
    assert_false(done);
    const decoder = new TextDecoder();
    messages.push(decoder.decode(value));
  }

  assert_array_equals(messages.sort(), ['message 1', 'message 2', 'message 3'],
                      'all messages received');
}, 'multiple writes through WebTransportDatagramsWritable work correctly');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable();
  const writer = writable.getWriter();

  await writer.ready;
  assert_equals(writer.desiredSize, 5, 'writer is ready to write');
}, 'WebTransportDatagramsWritable writer.ready works correctly');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  wt.closed.catch(() => {});
  await wt.ready;

  wt.close();
  await wt.closed;

  assert_throws_dom('InvalidStateError', () => {
    wt.datagrams.createWritable();
  }, 'createWritable should throw InvalidStateError on closed transport');
}, 'createWritable throws InvalidStateError on closed transport');

promise_test(async t => {
  const wt1 = new WebTransport(webtransport_url('echo.py'));
  wt1.closed.catch(() => {});
  t.add_cleanup(() => wt1.close());
  await wt1.ready;

  const wt2 = new WebTransport(webtransport_url('echo.py'));
  wt2.closed.catch(() => {});
  t.add_cleanup(() => wt2.close());
  await wt2.ready;

  const sendGroupFromWt1 = wt1.createSendGroup();

  assert_throws_dom('InvalidStateError', () => {
    wt2.datagrams.createWritable({ sendGroup: sendGroupFromWt1 });
  }, 'createWritable should throw InvalidStateError when sendGroup is from different transport');
}, 'createWritable throws InvalidStateError when sendGroup is from different transport');

promise_test(async t => {
  // NOTE: aioquic doesn't handle sending large datagrams well, and doesn't expose
  // what size if can handle.  It will queue large datagrams forever, and if
  // there are ACK frames or other data, even a max-size-for-aioquic
  // datagram will not be sent.  So we don't use echo.py here
  const wt = new WebTransport(webtransport_url('echo_datagram_length.py'));
  wt.closed.catch(() => {});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const writable = wt.datagrams.createWritable();
  const writer = writable.getWriter();
  const reader = wt.datagrams.readable.getReader();

  const maxSize = wt.datagrams.maxDatagramSize;
  writer.write(new Uint8Array(maxSize)).catch(() => {});

  // the server should echo the datagram length encoded in JSON
  const { value: token, done } = await reader.read();
  assert_false(done);

  const decoder = new TextDecoder();
  const datagramStr = decoder.decode(token);
  const jsonObject = JSON.parse(datagramStr);
  assert_equals(jsonObject['length'], maxSize);
}, 'WebTransportDatagramsWritable can write max-size datagrams');
