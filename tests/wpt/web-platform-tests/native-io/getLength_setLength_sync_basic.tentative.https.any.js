// META: title=Synchronous NativeIO API: Assigned length is observed back.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const writtenBytes = Uint8Array.from([97, 98, 99, 100]);
  file.write(writtenBytes, 0);

  file.setLength(3);
  const readBytes = readIoFileSync(file);

  const remainingBytes = Uint8Array.from([97, 98, 99]);
  assert_array_equals(
      readBytes, remainingBytes,
      'NativeIOFileSync.setLength() should remove bytes from the end of ' +
        'a file when decreasing its length.');
}, 'NativeIOFileSync.setLength shrinks a file and' +
     ' NativeIOFileSync.getLength reports its new length.');

test(testCase => {
  const file = storageFoundation.openSync('test_file');
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync('test_file');
  });

  const writtenBytes = Uint8Array.from([97, 98, 99, 100]);
  file.write(writtenBytes, 0);

  file.setLength(5);
  const readBytes = readIoFileSync(file);

  const expectedBytes = Uint8Array.from([97, 98, 99, 100, 0]);

  assert_array_equals(
      readBytes, expectedBytes,
      'NativeIOFileSync.setLength() should append zeros when increasing' +
        ' the file size.');
}, 'NativeIOFileSync.setLength appends zeros to a file and ' +
     'NativeIOFileSync.getLength reports its new length.');
