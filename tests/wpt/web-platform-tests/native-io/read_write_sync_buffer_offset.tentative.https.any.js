// META: title=NativeIO API: Read/write correctly offsets into the source buffer
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const writeBufferData = new Uint8Array([64, 65, 66, 67, 68, 69, 70, 71]);
  const writeBufferSource = new Uint8Array(writeBufferData.buffer, 3, 4);
  assert_equals(writeBufferSource.byteOffset, 3, 'Test setup error');
  assert_equals(writeBufferSource.byteLength, 4, 'Test setup error');
  assert_equals(writeBufferSource.buffer.byteLength, 8, 'Test setup error');

  const {buffer: writeBuffer, writtenBytes} = file.write(writeBufferSource, 0);

  assert_equals(
      writtenBytes, 4,
      'NativeIOFileSync.write() should return the number of bytes written');
  assert_equals(
      writeBuffer.byteOffset, 3,
      'NativeIOFileSync.write() should return the input view offset');
  assert_equals(
      writeBuffer.byteLength, 4,
      'NativeIOFileSync.write() should return the input view size');
  assert_equals(
      writeBuffer.buffer.byteLength, 8,
      'NativeIOFileSync.write() should use the same buffer as its input');

  const {buffer: readBuffer, readBytes} = file.read(new Uint8Array(4), 0);
  assert_equals(
      readBytes, 4,
      'NativeIOFileSync.write() should write the entire view size');

  assert_array_equals(
      readBuffer, [67, 68, 69, 70],
      'NativeIOFileSync.write() should account for the buffer offset');
}, 'NativeIOFileSync.write with a Uint8Array accounts for the buffer offset');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const writeBufferData = new Uint16Array(
      [0x4041, 0x4243, 0x4445, 0x4647, 0x4849, 0x4a4b, 0x4c4d, 0x4e4f]);
  const writeBufferSource = new Uint16Array(writeBufferData.buffer, 6, 4);
  assert_equals(writeBufferSource.byteOffset, 6, 'Test setup error');
  assert_equals(writeBufferSource.byteLength, 8, 'Test setup error');
  assert_equals(writeBufferSource.buffer.byteLength, 16, 'Test setup error');

  const {buffer: writeBuffer, writtenBytes} = file.write(writeBufferSource, 0);

  assert_equals(
      writtenBytes, 8,
      'NativeIOFileSync.write() should return the number of bytes written');
  assert_equals(
      writeBuffer.byteOffset, 6,
      'NativeIOFileSync.write() should return the input view offset');
  assert_equals(
      writeBuffer.byteLength, 8,
      'NativeIOFileSync.write() should return the input view size');
  assert_equals(
      writeBuffer.buffer.byteLength, 16,
      'NativeIOFileSync.write() should use the same buffer as its input');

  const {buffer: readBuffer, readBytes} = file.read(new Uint16Array(4), 0);
  assert_equals(
      readBytes, 8,
      'NativeIOFile.write() should write the entire view size');

  assert_array_equals(
      readBuffer, [0x4647, 0x4849, 0x4a4b, 0x4c4d],
      'NativeIOFileSync.write() should account for the buffer offset');
}, 'NativeIOFileSync.write with a Uint16Array accounts for the buffer offset');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const {buffer: writeBuffer, writtenBytes} =
      file.write(new Uint8Array([64, 65, 66, 67]), 0);
  assert_equals(
      writtenBytes, 4,
      'NativeIOFileSync.write() should return the number of bytes written');

  const readBufferData = new Uint8Array(8);
  const readBufferSource = new Uint8Array(readBufferData.buffer, 3, 4);
  assert_equals(readBufferSource.byteOffset, 3, 'Test setup error');
  assert_equals(readBufferSource.byteLength, 4, 'Test setup error');
  assert_equals(readBufferSource.buffer.byteLength, 8, 'Test setup error');

  const {buffer: readBuffer, readBytes} = file.read(readBufferSource, 0);
  assert_equals(
      readBytes, 4,
      'NativeIOFileSync.read() should read the entire input view size');
  assert_equals(
      readBuffer.byteOffset, 3,
      'NativeIOFileSync.read() should return the input view offset');
  assert_equals(
      readBuffer.byteLength, 4,
      'NativeIOFileSync.read() should return the input view size');
  assert_equals(
      readBuffer.buffer.byteLength, 8,
      'NativeIOFileSync.read() should use the same buffer as its input');

  assert_array_equals(
      readBuffer, writeBuffer,
      'NativeIOFileSync.read() should account for the buffer offset');

  const readBufferFull = new Uint8Array(readBuffer.buffer);
  assert_array_equals(
      readBufferFull, [0, 0, 0, 64, 65, 66, 67, 0],
      'NativeIOFileSync.read() should leave the buffer outside the view ' +
      'untouched');
}, 'NativeIOFileSync.read with a Uint8Array accounts for the buffer offset');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const {buffer: writeBuffer, writtenBytes} =
      file.write(new Uint16Array([0x4041, 0x4243, 0x4445, 0x4647]), 0);
  assert_equals(
      writtenBytes, 8,
      'NativeIOFileSync.write() should return the number of bytes written');

  const readBufferData = new Uint16Array(8);
  const readBufferSource = new Uint16Array(readBufferData.buffer, 6, 4);
  assert_equals(readBufferSource.byteOffset, 6, 'Test setup error');
  assert_equals(readBufferSource.byteLength, 8, 'Test setup error');
  assert_equals(readBufferSource.buffer.byteLength, 16, 'Test setup error');

  const {buffer: readBuffer, readBytes} = file.read(readBufferSource, 0);
  assert_equals(
      readBytes, 8,
      'NativeIOFileSync.read() should read the entire input view size');
  assert_equals(
      readBuffer.byteOffset, 6,
      'NativeIOFileSync.read() should return the input view offset');
  assert_equals(
      readBuffer.byteLength, 8,
      'NativeIOFileSync.read() should return the input view size');
  assert_equals(
      readBuffer.buffer.byteLength, 16,
      'NativeIOFileSync.read() should use the same buffer as its input');

  assert_array_equals(
      readBuffer, writeBuffer,
      'NativeIOFileSync.read() should account for the buffer offset');

  const readBufferFull = new Uint16Array(readBuffer.buffer);
  assert_array_equals(
      readBufferFull, [0, 0, 0, 0x4041, 0x4243, 0x4445, 0x4647, 0],
      'NativeIOFileSync.read() should leave the buffer outside the view ' +
      'untouched');
}, 'NativeIOFileSync.read with a Uint16Array accounts for the buffer offset');
