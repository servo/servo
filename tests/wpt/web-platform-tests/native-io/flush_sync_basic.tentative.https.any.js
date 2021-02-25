// META: title=Synchronous NativeIO API: Flushed data is read back.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const size = 1024;
  const longarray = createLargeArray(size, /*seed = */ 107);
  const writtenBytes = Uint8Array.from(longarray);
  const writeCount = file.write(writtenBytes, 0);
  assert_equals(
      writeCount, size,
      'NativeIOFile.write() should resolve with the number of bytes written');

  file.flush();
  const readBytes = readIoFileSync(file);

  assert_array_equals(readBytes, writtenBytes,
                      'the bytes read should match the bytes written');
}, 'NativeIOFileSync.read returns bytes written by NativeIOFileSync.write' +
     ' after NativeIOFileSync.flush');
