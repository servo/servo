'use strict';

directory_test(async (t, root_dir) => {
  const inboxBucket = await navigator.storageBuckets.open('inbox');
  const inboxRootDir = await inboxBucket.getDirectory();

  assert_false(await inboxRootDir.isSameEntry(root_dir));

  const handle1 = await createEmptyFile(t, 'mtime.txt', inboxRootDir);
  const handle2 = await inboxRootDir.getFileHandle('mtime.txt');
  assert_true(await handle1.isSameEntry(handle2));
}, 'isSameEntry works as expected with buckets');

directory_test(async (t, root_dir) => {
  const inboxBucket = await navigator.storageBuckets.open('inbox');
  await navigator.storageBuckets.delete('inbox');
  const directoryPromise = inboxBucket.getDirectory();
  await promise_rejects_dom(t, 'InvalidStateError', directoryPromise);
}, 'getDirectory promise rejects if bucket has been deleted');
