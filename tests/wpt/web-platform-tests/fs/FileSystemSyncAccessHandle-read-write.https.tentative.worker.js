importScripts("/resources/testharness.js");
importScripts('resources/sync-access-handle-test.js');

'use strict';

sync_access_handle_test((t, handle) => {
  const readBuffer = new Uint8Array(24);
  const readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(0, readBytes, 'Check that no bytes were read');
}, 'Test reading an empty file through a sync access handle.');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }

  const decoder = new TextDecoder();

  const text = 'Hello Storage Foundation';
  const writeBuffer = new TextEncoder().encode(text);
  const writtenBytes = handle.write(writeBuffer, {at: 0});
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');
  let readBuffer = new Uint8Array(writtenBytes);
  let readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(writtenBytes, readBytes, 'Check that all bytes were read');
  assert_equals(
      text, decoder.decode(readBuffer),
      'Check that the written bytes and the read bytes match');

  // Test a read of less bytes than available.
  const expected = 'Storage';
  readBuffer = new Uint8Array(expected.length);
  readBytes = handle.read(readBuffer, {at: text.indexOf(expected)});
  assert_equals(readBuffer.length, readBytes, 'Check that all bytes were read');
  const actual = decoder.decode(readBuffer);
  assert_equals(
      expected, actual,
      'Partial read returned unexpected contents');
}, 'Test writing and reading through a sync access handle.');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  for (text of ['Hello', 'Longer Text']) {
    const writeBuffer = encoder.encode(text);
    const writtenBytes = handle.write(writeBuffer, {at: 0});
    assert_equals(
        writeBuffer.byteLength, writtenBytes,
        'Check that all bytes were written.');
    const readBuffer = new Uint8Array(writtenBytes);
    const readBytes = handle.read(readBuffer, {at: 0});
    assert_equals(writtenBytes, readBytes, 'Check that all bytes were read');
    assert_equals(
        text, decoder.decode(readBuffer),
        'Check that the written bytes and the read bytes match');
  }
}, 'Test second write that is bigger than the first write');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  for (tuple
           of [{input: 'Hello World', expected: 'Hello World'},
               {input: 'foobar', expected: 'foobarWorld'}]) {
    const text = tuple.input;
    const expected = tuple.expected;
    const writeBuffer = encoder.encode(text);
    const writtenBytes = handle.write(writeBuffer, {at: 0});
    assert_equals(
        writeBuffer.byteLength, writtenBytes,
        'Check that all bytes were written.');
    const readBuffer = new Uint8Array(expected.length);
    const readBytes = handle.read(readBuffer, {at: 0});
    assert_equals(expected.length, readBytes, 'Check that all bytes were read');
    assert_equals(
        expected, decoder.decode(readBuffer),
        'Check that the written bytes and the read bytes match');
  }
}, 'Test second write that is smaller than the first write');

sync_access_handle_test((t, handle) => {
  const expected = 17;
  const writeBuffer = new Uint8Array(1);
  writeBuffer[0] = expected;
  const offset = 5;
  const writtenBytes = handle.write(writeBuffer, {at: offset});
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');
  const fileLength = writeBuffer.byteLength + offset;
  const readBuffer = new Uint8Array(fileLength);
  const readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(fileLength, readBytes, 'Check that all bytes were read');
  for (let i = 0; i < offset; ++i) {
    assert_equals(
        readBuffer[i], 0,
        `Gaps in the file should be filled with 0, but got ${readBuffer[i]}.`);
  }

  assert_equals(
      readBuffer[offset], expected,
      'Gaps in the file should be filled with 0.');
}, 'Test initial write with an offset');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  for (tuple
           of [{input: 'Hello World', expected: 'Hello World', offset: 0},
               {input: 'foobar', expected: 'Hello foobar', offset: 6}]) {
    const text = tuple.input;
    const expected = tuple.expected;
    const offset = tuple.offset;
    const writeBuffer = encoder.encode(text);
    const writtenBytes = handle.write(writeBuffer, {at: offset});
    assert_equals(
        writeBuffer.byteLength, writtenBytes,
        'Check that all bytes were written.');
    const readBuffer = new Uint8Array(expected.length);
    const readBytes = handle.read(readBuffer, {at: 0});
    assert_equals(expected.length, readBytes, 'Check that all bytes were read');
    const actual = decoder.decode(readBuffer);
    assert_equals(
        expected, actual,
        'Check content read from the handle');
  }
}, 'Test overwriting the file at an offset');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }

  const decoder = new TextDecoder();

  const text = 'Hello Storage Foundation';
  const writeBuffer = new TextEncoder().encode(text);
  const writtenBytes = handle.write(writeBuffer, {at: 0});
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');
  const bufferLength = text.length;
  for (tuple
           of [{offset: 0, expected: text},
               {offset: 6, expected: text.substring(6)}]) {
    const offset = tuple.offset;
    const expected = tuple.expected;

    const readBuffer = new Uint8Array(bufferLength);
    const readBytes = handle.read(readBuffer, {at: offset});
    assert_equals(expected.length, readBytes, 'Check that all bytes were read');
    const actual = decoder.decode(readBuffer);
    assert_true(
        actual.startsWith(expected),
        `Expected to read ${expected} but the actual value was ${actual}.`);
  }

  const readBuffer = new Uint8Array(bufferLength);
  // Offset is greater than the file length.
  const readBytes = handle.read(readBuffer, {at: bufferLength + 1});
  assert_equals(0, readBytes, 'Check that no bytes were read');
  for (let i = 0; i < readBuffer.byteLength; ++i) {
    assert_equals(0, readBuffer[i], 'Check that the read buffer is unchanged.');
  }
}, 'Test read at an offset');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }

  const expected = 'Hello Storage Foundation';
  const writeBuffer = new TextEncoder().encode(expected);
  const writtenBytes = handle.write(writeBuffer, {at: 0});
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');

  const bufferLength = expected.length;
  const readBuffer = new Uint8Array(expected.length);
  const readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(expected.length, readBytes, 'Check that all bytes were read');
  const actual = new TextDecoder().decode(readBuffer);
  assert_equals(
      expected, actual,
      `Expected to read ${expected} but the actual value was ${actual}.`);
}, 'Test read with default options');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }

  const expected = 'Hello Storage Foundation';
  const writeBuffer = new TextEncoder().encode(expected);
  // No options parameter provided, should write at offset 0.
  const writtenBytes = handle.write(writeBuffer);
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');

  const bufferLength = expected.length;
  const readBuffer = new Uint8Array(expected.length);
  const readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(expected.length, readBytes, 'Check that all bytes were read');
  const actual = new TextDecoder().decode(readBuffer);
  assert_equals(
      expected, actual,
      `Expected to read ${expected} but the actual value was ${actual}.`);
}, 'Test write with default options');

sync_access_handle_test((t, handle) => {
  const readBuffer = new Uint8Array(24);
  assert_throws_js(TypeError, () => handle.read(readBuffer, {at: -1}));
}, 'Test reading at a negative offset fails.');

sync_access_handle_test((t, handle) => {
  const text = 'foobar';
  const writeBuffer = new TextEncoder().encode(text);
  assert_throws_js(TypeError, () => handle.write(writeBuffer, {at: -1}));

  const readBuffer = new Uint8Array(24);
  const readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(0, readBytes, 'Check that no bytes were written');
}, 'Test writing at a negative offset fails.');

sync_access_handle_test((t, handle) => {
  const text = 'foobar';
  const writeBuffer = new TextEncoder().encode(text);
  let writtenBytes = handle.write(writeBuffer);
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');

  // This should double the file size, not overwrite the existing bytes.
  writtenBytes = handle.write(writeBuffer);
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');

  // Reading at the cursor will read zero bytes at the end of the file.
  const expectedFileSize = text.length * 2;
  const readBuffer = new Uint8Array(expectedFileSize);
  let readBytes = handle.read(readBuffer);
  assert_equals(0, readBytes, 'Check that no bytes were read');

  // Reading from the start of the file.
  readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(expectedFileSize, readBytes, 'Check that all bytes were read');
}, 'Test that writing moves the file position cursor');

sync_access_handle_test((t, handle) => {
  const decoder = new TextDecoder();

  const text = 'foobar';
  const writeBuffer = new TextEncoder().encode(text);
  let writtenBytes = handle.write(writeBuffer);
  assert_equals(
      writeBuffer.byteLength, writtenBytes,
      'Check that all bytes were written.');

  // Read the first half of the file.
  let expected = 'foo';
  const readBuffer = new Uint8Array(3);
  let readBytes = handle.read(readBuffer, {at: 0});
  assert_equals(3, readBytes, 'Check that all bytes were read');
  let actual = decoder.decode(readBuffer);
  assert_equals(
      expected, actual,
      `Expected to read ${expected} but the actual value was ${actual}.`);

  // Read the second half of the file, without specifying an offset.
  expected = 'bar';
  readBytes = handle.read(readBuffer);
  assert_equals(3, readBytes, 'Check that all bytes were read');
  actual = decoder.decode(readBuffer);
  assert_equals(
      expected, actual,
      `Expected to read ${expected} but the actual value was ${actual}.`);
}, 'Test that reading moves the file position cursor');

done();
