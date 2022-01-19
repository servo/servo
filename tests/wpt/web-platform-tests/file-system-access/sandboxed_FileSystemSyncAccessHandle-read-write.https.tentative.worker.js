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
  const encoder = new TextEncoder();
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
      text, new TextDecoder().decode(readBuffer),
      'Check that the written bytes and the read bytes match');

  // Test a read of less bytes than available.
  const expected = 'Storage';
  readBuffer = new Uint8Array(expected.length);
  readBytes = handle.read(readBuffer, {at: text.indexOf(expected)});
  assert_equals(readBuffer.length, readBytes, 'Check that all bytes were read');
  const actual = new TextDecoder().decode(readBuffer);
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
    const writeBuffer = new TextEncoder().encode(text);
    const writtenBytes = handle.write(writeBuffer, {at: 0});
    assert_equals(
        writeBuffer.byteLength, writtenBytes,
        'Check that all bytes were written.');
    const readBuffer = new Uint8Array(writtenBytes);
    const readBytes = handle.read(readBuffer, {at: 0});
    assert_equals(writtenBytes, readBytes, 'Check that all bytes were read');
    assert_equals(
        text, new TextDecoder().decode(readBuffer),
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
    const writeBuffer = new TextEncoder().encode(text);
    const writtenBytes = handle.write(writeBuffer, {at: 0});
    assert_equals(
        writeBuffer.byteLength, writtenBytes,
        'Check that all bytes were written.');
    const readBuffer = new Uint8Array(expected.length);
    const readBytes = handle.read(readBuffer, {at: 0});
    assert_equals(expected.length, readBytes, 'Check that all bytes were read');
    assert_equals(
        expected, new TextDecoder().decode(readBuffer),
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
    const writeBuffer = new TextEncoder().encode(text);
    const writtenBytes = handle.write(writeBuffer, {at: offset});
    assert_equals(
        writeBuffer.byteLength, writtenBytes,
        'Check that all bytes were written.');
    const readBuffer = new Uint8Array(expected.length);
    const readBytes = handle.read(readBuffer, {at: 0});
    assert_equals(expected.length, readBytes, 'Check that all bytes were read');
    const actual = new TextDecoder().decode(readBuffer);
    assert_equals(
        expected, actual,
        'Check content read from the handle');
  }
}, 'Test overwriting the file at an offset');

sync_access_handle_test((t, handle) => {
  if (!('TextEncoder' in self)) {
    return;
  }
  const encoder = new TextEncoder();
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
    const actual = new TextDecoder().decode(readBuffer);
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
  const readBuffer = new Uint8Array(24);
  assert_throws_dom(
    'NotSupportedError', () => handle.read(readBuffer, { at: -1 }));
}, 'Test reading at a negative offset fails.');

done();
