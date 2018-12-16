'use strict';

// This file depends on RTCQuicTransport-helper.js which should be loaded from
// the main HTML file.
// The following helper methods are called from RTCQuicTransport-helper.js:
//   makeTwoConnectedQuicTransports

// Run a test function for as many ways as an RTCQuicStream can transition to
// the 'closed' state.
// |test_func| will be called with the test as the first argument and the closed
//     RTCQuicStream as the second argument.
function closed_stream_test(test_func, description) {
  promise_test(async t => {
    const [ localQuicTransport, remoteQuicTransport ] =
        await makeTwoConnectedQuicTransports(t);
    const localStream = localQuicTransport.createStream();
    localStream.reset();
    assert_equals(localStream.state, 'closed');
    return test_func(t, localStream);
  }, 'Stream closed by local reset(): ' + description);

  promise_test(async t => {
    const [ localQuicTransport, remoteQuicTransport ] =
        await makeTwoConnectedQuicTransports(t);
    const localStream = localQuicTransport.createStream();
    localStream.write({ data: new Uint8Array(1) });
    const remoteWatcher =
        new EventWatcher(t, remoteQuicTransport, 'quicstream');
    const { stream: remoteStream } = await remoteWatcher.wait_for('quicstream');
    localStream.reset();
    const remoteStreamWatcher =
        new EventWatcher(t, remoteStream, 'statechange');
    await remoteStreamWatcher.wait_for('statechange');
    assert_equals(remoteStream.state, 'closed');
    return test_func(t, remoteStream);
  }, 'Stream closed by remote reset(): ' + description);

  promise_test(async t => {
    const [ localQuicTransport, remoteQuicTransport ] =
        await makeTwoConnectedQuicTransports(t);
    const localStream = localQuicTransport.createStream();
    localStream.write({ finish: true });
    const remoteWatcher =
        new EventWatcher(t, remoteQuicTransport, 'quicstream');
    const { stream: remoteStream } = await remoteWatcher.wait_for('quicstream');
    remoteStream.write({ finish: true });
    await localStream.waitForReadable(localStream.maxReadBufferedAmount);
    assert_object_equals(
        localStream.readInto(new Uint8Array(10)),
        { amount: 0, finished: true });
    assert_equals(localStream.state, 'closed');
    return test_func(t, localStream);
  }, 'Stream closed by writing a finish, followed by reading remote finish: ' +
      description);

  promise_test(async t => {
    const [ localQuicTransport, remoteQuicTransport ] =
        await makeTwoConnectedQuicTransports(t);
    const localStream = localQuicTransport.createStream();
    localStream.write({ finish: true });
    const remoteWatcher =
        new EventWatcher(t, remoteQuicTransport, 'quicstream');
    const { stream: remoteStream } = await remoteWatcher.wait_for('quicstream');
    await remoteStream.waitForReadable(10);
    assert_object_equals(
        remoteStream.readInto(new Uint8Array(10)),
        { amount: 0, finished: true });
    remoteStream.write({ finish: true });
    assert_equals(remoteStream.state, 'closed');
    return test_func(t, remoteStream);
  }, 'Stream closed by by reading remote finish, followed by writing a ' +
      'finish: ' + description);

  promise_test(async t => {
    const [ localQuicTransport, remoteQuicTransport ] =
        await makeTwoConnectedQuicTransports(t);
    const localStream = localQuicTransport.createStream();
    localQuicTransport.stop();
    assert_equals(localStream.state, 'closed');
    return test_func(t, localStream);
  }, 'Stream closed by local RTCQuicTransport stop(): ' + description);

  promise_test(async t => {
    const [ localQuicTransport, remoteQuicTransport ] =
        await makeTwoConnectedQuicTransports(t);
    const localStream = localQuicTransport.createStream();
    localStream.write({ data: new Uint8Array(1) });
    const remoteWatcher =
        new EventWatcher(t, remoteQuicTransport,
            [ 'quicstream', 'statechange' ]);
    const { stream: remoteStream } = await remoteWatcher.wait_for('quicstream');
    localQuicTransport.stop();
    await remoteWatcher.wait_for('statechange');
    assert_equals(remoteStream.state, 'closed');
    return test_func(t, remoteStream);
  }, 'Stream closed by remote RTCQuicTransport stop(): ' + description);
}

