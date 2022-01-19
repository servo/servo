// META: title=NativeIO API: Concurrent io while setLength is resolving.
// META: global=window,worker
// META: script=operation_helpers.js
// META: script=../resources/support.js
// META: timeout=long

'use strict';

// See documentation in operation_helpers.js

for (let op of kOperations) {
  promise_test(async testCase => {
    await reserveAndCleanupCapacity(testCase);

    const file = await createFile(testCase, 'setlength_file');

    const res = op.prepare();

    const setLengthPromise = file.setLength(5);
    await op.assertRejection(testCase, file, res);

    await setLengthPromise;

    const {buffer: readBuffer, readBytes} =
      await file.read(new Uint8Array(5), 0);

    assert_equals(readBytes, 5,
      `NativeIOFile.read() should not fail after a rejected ` +
      `${op.name}() during setLength().`);
    assert_array_equals(readBuffer, [64, 65, 66, 67, 0],
      `Rejecting ${op.name}() during setLength()` +
      ` should not change the file.`);
    op.assertUnchanged(res);
  }, `${op.name}() rejects while setLength() is resolving.`);
};
