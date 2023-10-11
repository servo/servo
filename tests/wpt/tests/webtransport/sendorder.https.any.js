// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js


promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a bidirectional stream with sendorder
  const {readable, writable} = await wt.createBidirectionalStream({sendOrder: 3});
  assert_equals(writable.sendOrder, 3);

  // Write a message to the writable end, and close it.
  const writer = writable.getWriter();
  const encoder = new TextEncoder();
  writer.write(encoder.encode('Hello World')).catch(() => {});
  await writer.close();

  // Read the data on the readable end.
  const reply = await read_stream_as_string(readable);

  // Check that the message from the readable end matches the writable end.
  assert_equals(reply, 'Hello World');
}, 'WebTransport client should be able to create and handle a bidirectional stream with sendOrder');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a bidirectional stream with sendorder
  const {readable, writable} = await wt.createBidirectionalStream();
  assert_equals(writable.sendOrder, null);
  // modify it
  writable.sendOrder = 4;
  assert_equals(writable.sendOrder, 4);
}, 'WebTransport client should be able to modify unset sendOrder after stream creation');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

    // Create a bidirectional stream without sendorder
  const {readable, writable} = await wt.createBidirectionalStream({sendOrder: 3});
  assert_equals(writable.sendOrder, 3);
  // modify it
  writable.sendOrder = 5;
  assert_equals(writable.sendOrder, 5);
  writable.sendOrder = null;
  assert_equals(writable.sendOrder, null);
  // Note: this doesn't verify the underlying stack actually changes priority, just the API
  // for controlling sendOrder
}, 'WebTransport client should be able to modify existing sendOrder after stream creation');

promise_test(async t => {
  // Establish a WebTransport session.
  const id = token();
  const wt = new WebTransport(webtransport_url(`sendorder.py?token=${id}`));
  await wt.ready;
  const bytes_low = new Uint8Array(65536).fill('1');
  const bytes_unordered = new Uint8Array(65536).fill('0');

  // Create a bidirectional stream without sendOrder
  const {writable: unordered_writable} = await wt.createBidirectionalStream();

  // Create a bidirectional stream with sendOrder
  const {writable: low_writable} = await wt.createBidirectionalStream({sendOrder: 1});

  // Write a large block to the lower-priority stream, async
  const low_writer = low_writable.getWriter();
  assert_equals(low_writable.sendOrder, 1);

  // Write a large block to the lower-priority stream, async
  const unordered_writer = unordered_writable.getWriter();

  // enough bytes written to ensure we'll fill the congestion window even
  // on a local server
  // this should be enough to require queuing
  for (let i = 0; i < 30; i++) {
    low_writer.write(bytes_low).catch(() => {});
  }
  for (let i = 0; i < 30; i++) {
    unordered_writer.write(bytes_unordered).catch(() => {});
  }

  await Promise.all([low_writer.close(), unordered_writer.close()]);

  // Read the data - first byte for each data reception
  const reply = await query(id);

  // If unordered data avoids starvation, some of it will come in before the end
  // of the sendordered data.
  // first packet normally will be '1', since that would likely
  // start being sent before unordered data starts queuing, but that's
  // not required by the spec, just that the unordered stream isn't starved
  //assert_equals(reply[0], 1);
  // Scan for the first 0 after we get a 1, then verify that more 1's come in after the first 0
  let ok = false;
  for (i = 0; i < reply.length; i++) {
    if (reply[i] == 1) {
      // scan for a 0
      for (; i < reply.length; i++) {
        if (reply[i] == 0) {
          for (; i < reply.length; i++) {
            if (reply[i] == 1) {
              // some unordered data came in before sendordered data, we're good
              ok = true;
              break;
            }
          }
          break;
        }
      }
      break;
    }
  }
  assert_true(ok);
}, 'WebTransport sendorder should not starve a stream without sendorder');

promise_test(async t => {
  // Establish a WebTransport session.
  const id = token();
  const wt = new WebTransport(webtransport_url(`sendorder.py?token=${id}`));
  await wt.ready;
  const bytes_low = new Uint8Array(65536).fill('1');
  const bytes_unordered = new Uint8Array(65536).fill('0');
  const bytes_high = new Uint8Array(65536).fill('2');

  // Create a bidirectional stream without sendOrder
  const {writable: unordered_writable} = await wt.createBidirectionalStream();

  // Create a bidirectional stream with sendOrder
  const {writable: low_writable} = await wt.createBidirectionalStream({sendOrder: 1});

  // Create a second bidirectional stream with higher sendOrder
  const {writable: high_writable} = await wt.createBidirectionalStream({sendOrder: 2});

  // Write a large block to the lower-priority stream, async
  const unordered_writer = unordered_writable.getWriter();

  // Write a large block to the lower-priority stream, async
  const low_writer = low_writable.getWriter();
  assert_equals(low_writable.sendOrder, 1);
  const high_writer = high_writable.getWriter();
  assert_equals(high_writable.sendOrder, 2);

  // enough bytes written to ensure we'll fill the congestion window even
  // on a local server
  // this should be enough to require queuing
  for (let i = 0; i < 30; i++) {
    unordered_writer.write(bytes_unordered).catch(() => {});
  }
  for (let i = 0; i < 30; i++) {
    low_writer.write(bytes_low).catch(() => {});
  }
  // these should jump the queue and get sent before the low-priority data finishes
  for (let i = 0; i < 30; i++) {
    high_writer.write(bytes_high).catch(() => {});
  }

  await Promise.all([low_writer.close(), unordered_writer.close(), high_writer.close()]);

  // Read the data - first byte for each data reception
  const reply = await query(id);

  // If high priority data gets prioritized, it won't be last received.  If
  // it isn't prioritized, it will likely come in after all the
  // low-priority data.  The first packet normally will be '0' (unordered),
  // since that would likely start being sent before low_priority data
  // shows up in the queue (and then they'll round-robin until
  // high-priority data gets queued). Some low-priority data will likely
  // start coming in before the high priority data gets queued; after high
  // priority data is queued it should jump ahead of the low-priority data
  // (and interleave with unordered).  Some low-priority data may
  // interleave with high-priority data, since it doesn't get queued all at
  // once.
  assert_true(reply[reply.length-1] != 2);
}, 'WebTransport sendorder should starve a lower priority stream');

promise_test(async t => {
  // Establish a WebTransport session.
  const id = token();
  const wt = new WebTransport(webtransport_url(`sendorder.py?token=${id}`));
  await wt.ready;
  const bytes_low = new Uint8Array(65536).fill('1');
  const bytes_unordered = new Uint8Array(65536).fill('0');
  const bytes_high = new Uint8Array(65536).fill('2');

  // Create a bidirectional stream without sendOrder
  const {writable: unordered_writable} = await wt.createBidirectionalStream();

  // Create a bidirectional stream with sendOrder
  const {writable: low_writable} = await wt.createBidirectionalStream({sendOrder: 1});

  // Create a second bidirectional stream with higher sendOrder
  const {writable: high_writable} = await wt.createBidirectionalStream({sendOrder: 2});

  // Write a large block to the lower-priority stream, async
  const unordered_writer = unordered_writable.getWriter();

  // Write a large block to the lower-priority stream, async
  const low_writer = low_writable.getWriter();
  assert_equals(low_writable.sendOrder, 1);
  const high_writer = high_writable.getWriter();
  assert_equals(high_writable.sendOrder, 2);

  // enough bytes written to ensure we'll fill the congestion window even
  // on a local server
  // this should be enough to require queuing
  for (let i = 0; i < 30; i++) {
    unordered_writer.write(bytes_unordered).catch(() => {});
  }
  // Alternate version where high-priority data should always come in
  // before low-priority, assuming we've saturated the output queue and can
  // feed data in faster than it goes through.
  for (let i = 0; i < 30; i++) {
    high_writer.write(bytes_high).catch(() => {});
  }
  for (let i = 0; i < 30; i++) {
    low_writer.write(bytes_low).catch(() => {});
  }

  await Promise.all([low_writer.close(), unordered_writer.close(), high_writer.close()]);

  // Read the data - first byte for each data reception
  const reply = await query(id);

  // Scan for the last 2, and verify there are no 1's before it
  let ok = true;
  for (i = 0; i < reply.length; i++) {
    if (reply[i] == 1) {
      // scan for a 2
      for (; i < reply.length; i++) {
        if (reply[i] == 2) {
          // priority 1 data should never jump in front of priority 2
          ok = false;
          break;
        }
      }
      break;
    }
  }
  assert_true(ok);
}, 'WebTransport sendorder should starve a lower priority stream, variant 2');


// XXX add tests for unordered vs ordered
