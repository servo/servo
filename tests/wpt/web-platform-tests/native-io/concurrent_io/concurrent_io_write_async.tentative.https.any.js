// META: title=NativeIO API: Concurrent IO while write is resolving.
// META: global=window,worker
// META: script=operation_helpers.js
// META: script=../resources/support.js
// META: timeout=long

'use strict';

// See documentation in operation_helpers.js

for (let op of kOperations) {
  promise_test(async testCase => {
    await reserveAndCleanupCapacity(testCase);

    const file = await createFile(testCase, 'write_file');

    const writeSharedArrayBuffer = new SharedArrayBuffer(4);
    const writtenBytes = new Uint8Array(writeSharedArrayBuffer);
    writtenBytes.set([96, 97, 98, 99]);
    const res = op.prepare();

    const writePromise = file.write(writtenBytes, 0);
    await op.assertRejection(testCase, file, res);

    assert_equals(await writePromise, 4);

    const readSharedArrayBuffer = new SharedArrayBuffer(4);
    const readBytes = new Uint8Array(readSharedArrayBuffer);
    assert_equals(await file.read(readBytes, 0), 4,
                  `NativeIOFile.read() should not fail after a rejected ` +
                    `${op.name} during write()`);
    assert_array_equals(readBytes, writtenBytes,
                        `Rejecting ${op.name} during write() should still ` +
                          `complete the write.`);
    op.assertUnchanged(res);
  }, `${op.name}() rejects while write() is resolving.`);
};
