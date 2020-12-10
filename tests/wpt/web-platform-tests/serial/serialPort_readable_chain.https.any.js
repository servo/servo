// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  // Select a buffer size larger than the amount of data transferred.
  await port.open({baudRate: 9600, bufferSize: 64});

  const decoder = new TextDecoderStream();
  const streamClosed = port.readable.pipeTo(decoder.writable);
  const readable = decoder.readable.pipeThrough(new TransformStream())
                       .pipeThrough(new TransformStream())
                       .pipeThrough(new TransformStream())
                       .pipeThrough(new TransformStream());
  const reader = readable.getReader();

  await fakePort.writable();
  fakePort.write(new TextEncoder().encode('Hello world!'));

  const {value, done} = await reader.read();
  assert_false(done);
  assert_equals('Hello world!', value);
  await reader.cancel('arbitrary reason');
  await streamClosed.catch(reason => {
    assert_equals('arbitrary reason', reason);
  });

  await port.close();
}, 'Stream closure is observable through a long chain of transforms');
