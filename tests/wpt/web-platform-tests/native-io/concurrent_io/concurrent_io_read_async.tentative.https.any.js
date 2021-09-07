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

    const res = op.prepare();

    const readPromise = file.read(new Uint8Array(4), 0);
    await op.assertRejection(testCase, file, res);

    let {buffer: readBuffer, readBytes} = await readPromise;
    assert_equals(readBytes, 4);
    assert_array_equals(readBuffer, [64, 65, 66, 67]);

    readBuffer.fill(0);

    ({buffer: readBuffer, readBytes} = await file.read(readBuffer, 0));
    assert_equals(readBytes, 4,
                  'NativeIOFile.read() should not fail after a rejected ' +
                    `${op.name} during read()`);
    assert_array_equals(readBuffer, [64, 65, 66, 67],
                        `Rejecting ${op.name} during read() should not ` +
                          'change the file.');
    op.assertUnchanged(res);
  }, `${op.name}() rejects while read() is resolving.`);
};
