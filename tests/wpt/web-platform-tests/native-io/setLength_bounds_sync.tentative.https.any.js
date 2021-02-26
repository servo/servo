// META: title=Synchronous NativeIO API: Out-of-bounds errors for setLength.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, "file_length_zero");
  file.setLength(0);
  const lengthDecreased = file.getLength();
  assert_equals(lengthDecreased, 0,
                "NativeIOFileSync.setLength() should set the file length to 0.");
}, 'NativeIOFileSync.setLength() does not throw an error when descreasing ' +
     'the file length to 0.');

test(testCase => {
  reserveAndCleanupCapacitySync(testCase);

  const file = createFileSync(testCase, "file_length_negative");

  // Without this assertion, the test passes even if setLength is not defined.
  assert_implements(file.setLength,
                    "NativeIOFileSync.setLength is not implemented.");

  assert_throws_js(TypeError, () => file.setLength(-1));
}, 'NativeIOFileSync.setLength() throws when setting negative lengths.');
