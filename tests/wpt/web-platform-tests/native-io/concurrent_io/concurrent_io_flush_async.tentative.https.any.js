// META: title=NativeIO API: Concurrent io while flush is resolving.
// META: global=window,worker
// META: script=operation_helpers.js
// META: script=../resources/support.js


'use strict';

// See documentation in operation_helpers.js

for (let op of kOperations) {
  promise_test(async testCase => {
    await reserveAndCleanupCapacity(testCase);

    const file = await createFile(testCase, 'flush_file');

    const res = op.prepare();

    const flushPromise = file.flush();
    await op.assertRejection(testCase, file, res);

    await flushPromise;

    const readSharedArrayBuffer = new SharedArrayBuffer(4);
    const readBytes = new Uint8Array(readSharedArrayBuffer);
    assert_equals(await file.read(readBytes, 0), 4,
                  `NativeIOFile.read() should not fail after a rejected ` +
                    `${op.name}() during flush()`);
    assert_array_equals(readBytes, [64, 65, 66, 67],
                        `Rejecting ${op.name}() during flush() should not ` +
                          `change the file.`);
    op.assertUnchanged(res);
  }, `${op.name}() rejects while flush() is resolving.`);
};
