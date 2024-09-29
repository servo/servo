'use strict';

directory_test(async (t, root_dir) => {
  await prepareForBucketTest(t);

  const inboxBucket = await navigator.storageBuckets.open('inbox');
  const inboxRootDir = await inboxBucket.getDirectory();

  assert_false(await inboxRootDir.isSameEntry(root_dir));

  const handle1 = await createEmptyFile('mtime.txt', inboxRootDir);
  const handle2 = await inboxRootDir.getFileHandle('mtime.txt');
  assert_true(await handle1.isSameEntry(handle2));
}, 'isSameEntry works as expected with buckets');

directory_test(async (t, root_dir) => {
  await prepareForBucketTest(t);

  const inboxBucket = await navigator.storageBuckets.open('inbox');
  await navigator.storageBuckets.delete('inbox');
  const directoryPromise = inboxBucket.getDirectory();
  await promise_rejects_dom(t, 'InvalidStateError', directoryPromise);
}, 'getDirectory promise rejects if bucket has been deleted');

directory_test(async (t, root_dir) => {
  await prepareForBucketTest(t);

  const inboxBucket =
      await navigator.storageBuckets.open('inbox', {quota: 500});
  const inboxRootDir = await inboxBucket.getDirectory();

  // Short file succeeds.
  const file =
      await createFileWithContents('mtime.txt', 'short file', inboxRootDir);

  // Longer file fails.
  return promise_rejects_dom(
      t, 'QuotaExceededError',
      createFileWithContents(
          'mtime2.txt',
          'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum',
          inboxRootDir));
}, 'Bucket quota restricts the size of a file that can be created');
