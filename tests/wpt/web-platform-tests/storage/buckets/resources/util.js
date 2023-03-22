'use strict';

// Makes sure initial bucket state is as expected and to clean up after the test
// is over (whether it passes or fails).
async function prepareForBucketTest(test) {
  // Verify initial state.
  assert_equals('', (await navigator.storageBuckets.keys()).join());
  // Clean up after test.
  test.add_cleanup(async function() {
    const keys = await navigator.storageBuckets.keys();
    for (const key of keys) {
      await navigator.storageBuckets.delete(key);
    }
  });
}
