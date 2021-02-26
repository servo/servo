// META: title=NativeIO API: Concurrent IO while getLength is resolving.
// META: global=window,worker
// META: script=operation_helpers.js
// META: script=../resources/support.js

'use strict';

// See documentation in operation_helpers.js

for (let op of kOperations) {
  promise_test(async testCase => {
    await reserveAndCleanupCapacity(testCase);

    const file = await createFile(testCase, 'getlength_file');

    const res = op.prepare();

    const getLengthPromise = file.getLength();
    await op.assertRejection(testCase, file, res);

    assert_equals(await getLengthPromise, 4);

    const readSharedArrayBuffer = new SharedArrayBuffer(4);
    const readBytes = new Uint8Array(readSharedArrayBuffer);
    assert_equals(await file.read(readBytes, 0), 4,
                  `NativeIOFile.read() should not fail after a rejected ` +
                    `${op.name} during getLength()`);
    assert_array_equals(readBytes, [64, 65, 66, 67],
                        `Rejecting ${op.name} during getLength() should not ` +
                          `change the file.`);
    op.assertUnchanged(res);
  }, `${op.name}() rejects while getLength() is resolving.`);
};
