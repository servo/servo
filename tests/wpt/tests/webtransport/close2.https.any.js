// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const wt = new WebTransport(
      webtransport_url(
          'server-create-multiple-streams.py?type=bidi&count=2&marker=1'));
  add_completion_callback(() => wt.close());
  await wt.ready;

  // Incoming streams aren't ordered across types, but a marker the server
  // opens after the bidirectional streams usually arrives after them, leaving
  // them available but unread when close() runs.  Where an implementation
  // delivers streams in the order they were opened, the marker guarantees
  // this and the result is definitive.
  //
  // Where it doesn't, the marker can overtake the bidirectional streams, for
  // example if their packets are lost or reordered, or the implementation
  // surfaces each stream type through a separate path.  The pull during
  // cleanup then finds nothing to receive, and the test passes without
  // exercising the bug; it can't fail spuriously.  The server sends all three
  // streams back to back, so loss or reordering on a loopback connection
  // should be rare.  Separate per-type paths could lose the race more often,
  // but that only weakens the test for that implementation.
  const uniReader = wt.incomingUnidirectionalStreams.getReader();
  const {value: marker} = await uniReader.read();
  assert_equals(await read_stream_as_string(marker), 'marker');

  const reader = wt.incomingBidirectionalStreams.getReader();

  // Resolving closed with closeInfo looks up "then" on it synchronously,
  // which lets script pull from incomingBidirectionalStreams during cleanup.
  let readDuringCleanup;
  t.add_cleanup(() => { delete Object.prototype.then; });
  Object.defineProperty(Object.prototype, 'then', {
    configurable: true,
    get() {
      if ('closeCode' in this && 'reason' in this) {
        delete Object.prototype.then;
        readDuringCleanup = reader.read();
      }
      return undefined;
    },
  });
  wt.close();
  delete Object.prototype.then;

  assert_not_equals(readDuringCleanup, undefined,
                    'then getter should run while closed resolves');
  const {done} = await readDuringCleanup;
  assert_true(done, 'no stream should be produced after cleanup');
  await wt.closed;
}, 'pulling incomingBidirectionalStreams while closed resolves should not ' +
   'produce a stream');
