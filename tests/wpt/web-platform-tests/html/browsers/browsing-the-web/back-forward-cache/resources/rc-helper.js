// A collection of helper functions that make use of the `remoteContextHelper`
// to test BFCache support and behavior.

// Call `prepareForBFCache()` before navigating away from the page. This simply
// sets a variable in window.
async function prepareForBFCache(remoteContextHelper) {
  await remoteContextHelper.executeScript(() => {
    window.beforeBFCache = true;
  });
}

// Call `getBeforeCache()` after navigating back to the page. This returns the
// value in window.
async function getBeforeBFCache(remoteContextHelper) {
  return await remoteContextHelper.executeScript(() => {
    return window.beforeBFCache;
  });
}

// If the value in window is set to true, this means that the page was reloaded,
// i.e., the page was restored from BFCache.
// Call `prepareForBFCache()` before navigating away to call this function.
async function assert_implements_bfcache(remoteContextHelper) {
  var beforeBFCache = await getBeforeBFCache(remoteContextHelper);
  assert_implements_optional(beforeBFCache == true, 'BFCache not supported.');
}

// If the value in window is undefined, this means that the page was reloaded,
// i.e., the page was not restored from BFCache.
// Call `prepareForBFCache()` before navigating away to call this function.
async function assert_not_bfcached(remoteContextHelper) {
  var beforeBFCache = await getBeforeBFCache(remoteContextHelper);
  assert_equals(beforeBFCache, undefined);
}

// A helper function that combines the steps of setting window property,
// navigating away and back, and making assertion on whether BFCache is
// supported.
async function assertBFCache(remoteContextHelper, shouldRestoreFromBFCache) {
  await prepareForBFCache(remoteContextHelper);
  // Navigate away and back.
  const newRemoteContextHelper = await remoteContextHelper.navigateToNew();
  await newRemoteContextHelper.historyBack();

  if (shouldRestoreFromBFCache) {
    await assert_implements_bfcache(remoteContextHelper);
  } else {
    await assert_not_bfcached(remoteContextHelper);
  }
}
