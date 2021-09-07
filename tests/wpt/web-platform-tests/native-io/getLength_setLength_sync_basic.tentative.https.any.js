// META: title=Synchronous NativeIO API: Assigned length is observed back.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'test_file', [97, 98, 99, 100]);

  file.setLength(3);
  const readBuffer = readIoFileSync(file);

  const remainingBytes = Uint8Array.from([97, 98, 99]);
  assert_array_equals(
    readBuffer, remainingBytes,
    'NativeIOFileSync.setLength() should remove bytes from the end of ' +
    'a file when decreasing its length.');
}, 'NativeIOFileSync.setLength shrinks a file and' +
' NativeIOFileSync.getLength reports its new length.');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, 'test_file', [97, 98, 99, 100]);

  file.setLength(5);
  const readBuffer = readIoFileSync(file);

  const expectedBytes = Uint8Array.from([97, 98, 99, 100, 0]);

  assert_array_equals(
    readBuffer, expectedBytes,
    'NativeIOFileSync.setLength() should append zeros when increasing' +
    ' the file size.');
}, 'NativeIOFileSync.setLength appends zeros to a file and ' +
'NativeIOFileSync.getLength reports its new length.');
