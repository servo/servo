// META: script=/resources/test-only-api.js
// META: script=/serial/resources/common.js
// META: script=resources/automation.js

async function readInto(reader, buffer) {
  let offset = 0;
  while (offset < buffer.byteLength) {
    const {value: view, done} =
        await reader.read(new Uint8Array(buffer, offset));
    buffer = view.buffer;
    if (done) {
      break;
    }
    offset += view.byteLength;
  }
  return buffer;
}

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  const bufferSize = 1024;
  await port.open({baudRate: 9600, bufferSize});

  const reader = port.readable.getReader({mode: 'byob'});
  assert_true(reader instanceof ReadableStreamBYOBReader);

  await fakePort.writable();
  const data = new Uint8Array(bufferSize);
  for (let i = 0; i < data.byteLength; ++i)
    data[i] = i & 0xff;
  fakePort.write(data);

  let buffer = new ArrayBuffer(512);
  buffer = await readInto(reader, buffer);
  assert_equals(512, buffer.byteLength, 'original size retained');
  compareArrays(data.subarray(0, 512), new Uint8Array(buffer));

  buffer = await readInto(reader, buffer);
  assert_equals(512, buffer.byteLength, 'original size retained');
  compareArrays(data.subarray(512), new Uint8Array(buffer));
  reader.releaseLock();

  await port.close();
}, 'Can read specific subsets of the available data');

serial_test(async (t, fake) => {
  const {port, fakePort} = await getFakeSerialPort(fake);
  await port.open({baudRate: 9600, bufferSize: 64});

  const reader = port.readable.getReader({mode: 'byob'});
  assert_true(reader instanceof ReadableStreamBYOBReader);

  await fakePort.writable();
  const data = new Uint8Array(1024);
  for (let i = 0; i < data.byteLength; ++i)
    data[i] = i & 0xff;
  fakePort.write(data);

  let buffer = new ArrayBuffer(1024);
  buffer = await readInto(reader, buffer);
  compareArrays(data, new Uint8Array(buffer));
  reader.releaseLock();

  await port.close();
}, 'Can read a large amount of data over multiple iterations');
