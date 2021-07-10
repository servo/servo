// META: title=NativeIO API: Concurrent IO while read is resolving.
// META: global=window,worker
// META: script=operation_helpers.js
// META: script=../resources/support.js
// META: timeout=long

'use strict';

// See documentation in operation_helpers.js

for (let op of kOperations) {
  promise_test(async testCase => {
    await reserveAndCleanupCapacity(testCase);

    const file = await createFile(testCase, 'read_file');

    const readSharedArrayBuffer = new SharedArrayBuffer(4);
    const readBytes = new Uint8Array(readSharedArrayBuffer);
    const res = op.prepare();

    const readPromise = file.read(readBytes, 0);
    await op.assertRejection(testCase, file, res);

    assert_equals(await readPromise, 4);
    assert_array_equals(readBytes, [64, 65, 66, 67]);

    readBytes.fill(0);
    assert_equals(await file.read(readBytes, 0), 4,
                  'NativeIOFile.read() should not fail after a rejected ' +
                    `${op.name} during read()`);
    assert_array_equals(readBytes, [64, 65, 66, 67],
                        `Rejecting ${op.name} during read() should not ` +
                          'change the file.');
    op.assertUnchanged(res);
  }, `${op.name}() rejects while read() is resolving.`);
};
