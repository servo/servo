// META: title=Synchronous NativeIO API: Written bytes are read back.
// META: global=dedicatedworker

'use strict';

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const writtenBytes = Uint8Array.from([64, 65, 66, 67]);
  const writeCount = file.write(writtenBytes, 0);
  assert_equals(
      writeCount, 4,
      'NativeIOFile.write() should resolve with the number of bytes written');

  const readBytes = new Uint8Array(writtenBytes.length);
  const readCount = file.read(readBytes, 0);
  assert_equals(readCount, 4,
                'NativeIOFile.read() should return the number of bytes read');

  assert_array_equals(readBytes, writtenBytes,
                      'the bytes read should match the bytes written');
}, 'NativeIOFileSync.read returns bytes written by NativeIOFileSync.write');
