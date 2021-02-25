// META: title=Synchronous NativeIO API: getLength reports written bytes.
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
      'NativeIOFileSync.write() should resolve with the number of bytes' +
      'written.');

  const length = file.getLength();
  assert_equals(length, 4,
                'NativeIOFileSync.getLength() should return the number of' +
                ' bytes in the file.');
}, 'NativeIOFileSync.getLength returns the number bytes written by' +
    ' NativeIOFileSync.write');
