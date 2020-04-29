// META: title=Synchronous NativeIO API: close().
// META: global=dedicatedworker

'use strict';

// Returns a handle to a newly created file that holds some data.
//
// The file will be closed and deleted when the test ends.
function createFileSync(testCase, fileName) {
  const file = nativeIO.openSync(fileName);
  testCase.add_cleanup(() => {
    file.close();
    nativeIO.deleteSync('test_file');
  });

  const writtenBytes = Uint8Array.from([64, 65, 66, 67]);
  const writeCount = file.write(writtenBytes, 0);
  assert_equals(writeCount, 4);

  return file;
}

test(testCase => {
  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  assert_equals(undefined, file.close());
}, 'nativeIO.close is idempotent');

test(testCase => {
  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  const readBytes = new Uint8Array(4);
  assert_throws_dom('InvalidStateError', () => file.read(readBytes, 4));
}, 'nativeIO.read fails after nativeIO.close settles');

test(testCase => {
  const file = createFileSync(testCase, 'file_name');
  assert_equals(undefined, file.close());

  const writtenBytes = Uint8Array.from([96, 97, 98, 99]);
  assert_throws_dom('InvalidStateError', () => file.write(writtenBytes, 4));
}, 'NativeIOFile.write fails after NativeIOFile.close');
