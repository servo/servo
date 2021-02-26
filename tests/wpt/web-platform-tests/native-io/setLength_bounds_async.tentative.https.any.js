// META: title=NativeIO API: Out-of-bounds errors for setLength.
// META: global=window,worker
// META: script=resources/support.js

'use strict';

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, "file_length_zero");
  await file.setLength(0);
  const lengthDecreased = await file.getLength();
  assert_equals(lengthDecreased, 0,
                "NativeIOFile.setLength() should set the file length to 0.");
}, 'NativeIOFile.setLength does not throw an error when descreasing the ' +
     'file length to 0.');

promise_test(async testCase => {
  await reserveAndCleanupCapacity(testCase);

  const file = await createFile(testCase, "file_length_negative");
  await promise_rejects_js(testCase, TypeError,
                           file.setLength(-1));
}, 'NativeIOFile.setLength() throws when setting negative lengths.');
