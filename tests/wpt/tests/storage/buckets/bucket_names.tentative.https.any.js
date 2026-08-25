// META: title=Buckets API: Basic tests for bucket names.
// META: script=resources/util.js
// META: global=window,worker

const kGoodBucketNameTests = [
  ['abcdefghijklmnopqrstuvwxyz0123456789-_', 'with allowed characters'],
  ['2021-01-01', 'with `-` in the middle'],
  ['2021_01_01', 'with `_` in the middle'],
  ['2021_01_01_', 'ending with `_`'],
  ['2021-01-01-', 'ending with `-`'],
];

const kBadBucketNameTests = [
  ['_bucket', 'start with `_`'],
  ['-bucket', 'start with `-`'],
  ['bucket name', 'have a space'],
  ['bUcKet123', 'are not all lower case'],
  ['bucket♦♥♠♣', 'are not in ASCII'],
  ['2021/01/01', 'include an invalid special character'],
  ['   ', 'have no characters'],
  ['', 'are an empty string'],
  ['mjnkhtwsiyjsrxvrzzqafldfvomqopdjfiuxqelfkllcugrhvvblkvmiqlguhhqepoggyu',
   'exceed 64 chars']
];

// Test valid bucket names on open().
kGoodBucketNameTests.forEach(test_data => {
  const bucket_name = test_data[0];
  const test_description = test_data[1];

  promise_test(async testCase => {
    await prepareForBucketTest(testCase);
    const bucket = await navigator.storageBuckets.open(bucket_name);
    assert_equals(bucket.name, bucket_name);

    const buckets = await navigator.storageBuckets.keys();
    assert_array_equals(buckets, [bucket_name]);
  }, `open() allows bucket names ${test_description}`);
});

// Test invalid bucket names on open().
kBadBucketNameTests.forEach(test_data => {
  const bucket_name = test_data[0];
  const test_description = test_data[1];

  promise_test(async testCase => {
    await prepareForBucketTest(testCase);
    return promise_rejects_js(
        testCase, TypeError,
        navigator.storageBuckets.open(bucket_name));
  }, `open() throws an error if bucket names ${test_description}`);
});

// Test valid bucket names on delete().
kGoodBucketNameTests.forEach(test_data => {
  const bucket_name = test_data[0];
  const test_description = test_data[1];

  promise_test(async testCase => {
    await prepareForBucketTest(testCase);
    await navigator.storageBuckets.open(bucket_name);
    let buckets = await navigator.storageBuckets.keys();
    assert_equals(buckets.length, 1);

    await navigator.storageBuckets.delete(bucket_name);

    buckets = await navigator.storageBuckets.keys();
    assert_equals(buckets.length, 0);
  }, `delete() allows bucket names ${test_description}`);
});

// Test invalid bucket names on delete().
kBadBucketNameTests.forEach(test_data => {
  const bucket_name = test_data[0];
  const test_description = test_data[1];

  promise_test(async testCase => {
    await prepareForBucketTest(testCase);
    return promise_rejects_js(
        testCase, TypeError,
        navigator.storageBuckets.delete(bucket_name));
  }, `delete() throws an error if bucket names ${test_description}`);
});

promise_test(async testCase => {
  await prepareForBucketTest(testCase);

  await navigator.storageBuckets.open('bucket_name');
  await navigator.storageBuckets.open('bucket_name');

  const buckets = await navigator.storageBuckets.keys();
  assert_array_equals(buckets, ['bucket_name']);
}, 'open() does not store duplicate bucket names');
