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
async function assertImplementsBFCacheOptional(remoteContextHelper) {
  var beforeBFCache = await getBeforeBFCache(remoteContextHelper);
  assert_implements_optional(beforeBFCache == true, 'BFCache not supported.');
}

// Subtracts set `b` from set `a` and returns the result.
function setMinus(a, b) {
  const minus = new Set();
  a.forEach(e => {
    if (!b.has(e)) {
      minus.add(e);
    }
  });
  return minus;
}

// Return a sorted Array from the iterable `s`.
function sorted(s) {
  return Array.from(s).sort();
}

// Assert expected reasons are all present. Note that the extra reasons are allowed
// as UAs might block bfcache for their specific reasons.
function matchReasons(expectedNotRestoredReasonsSet, notRestoredReasonsSet) {
  const missing = setMinus(
    expectedNotRestoredReasonsSet, notRestoredReasonsSet);
  const extra = setMinus(
      notRestoredReasonsSet, expectedNotRestoredReasonsSet);
  assert_true(missing.size == 0, `Expected: ${sorted(expectedNotRestoredReasonsSet)}\n` +
    `Got: ${sorted(notRestoredReasonsSet)}\n` +
    `Missing: ${sorted(missing)}\n` +
    `Extra: ${sorted(extra)}\n`);
}

// This function takes a set of reasons and extracts reasons out of it and returns a set of strings.
// For example, if the input is [{"reason": "error-document"}, {"reason": "masked"}],
// the output is ["error-document", "masked"].
function extractReason(reasonSet) {
  let reasonsExtracted = new Set();
  for (let reason of reasonSet) {
    reasonsExtracted.add(reason.reason);
  }
  return reasonsExtracted;
}

// A helper function to assert that the page is not restored from BFCache by
// checking whether the `beforeBFCache` value from `window` is undefined
// due to page reload.
// This function also takes an optional `notRestoredReasons` list which
// indicates the set of expected reasons that make the page not restored.
// If the reasons list is undefined, the check will be skipped. Otherwise
// this check will use the `notRestoredReasons` API, to obtain the reasons
// in a tree structure, and flatten the reasons before making the order-
// insensitive comparison.
// If the API is not available, the function will terminate instead of marking
// the assertion failed.
// Call `prepareForBFCache()` before navigating away to call this function.
// `preconditionFailReasons` is a set of reasons that could be reported but
// should PRECONDITION_FAIL if so. If `preconditionFailReasons` are reported,
// this function will not check if `notRestoredReasons` are reported.
async function assertNotRestoredFromBFCache(
    remoteContextHelper, notRestoredReasons, preconditionFailReasons = null) {
  var beforeBFCache = await getBeforeBFCache(remoteContextHelper);
  assert_equals(beforeBFCache, undefined, 'document unexpectedly BFCached');

  // The reason is optional, so skip the remaining test if the
  // `notRestoredReasons` is not set.
  if (notRestoredReasons === undefined) {
    return;
  }

  let isFeatureEnabled = await remoteContextHelper.executeScript(() => {
    return 'notRestoredReasons' in
        performance.getEntriesByType('navigation')[0];
  });

  // Return if the `notRestoredReasons` API is not available.
  if (!isFeatureEnabled) {
    return;
  }

  let result = await remoteContextHelper.executeScript(() => {
    return performance.getEntriesByType('navigation')[0].notRestoredReasons;
  });

  let expectedNotRestoredReasonsSet = new Set(notRestoredReasons);
  let notRestoredReasonsSet = new Set();

  // Flatten the reasons from the main frame and all the child frames.
  const collectReason = (node) => {
    for (let reason of node.reasons) {
      notRestoredReasonsSet.add(reason.reason);
    }
    for (let child of node.children) {
      collectReason(child);
    }
  };
  collectReason(result);

  // Check for preconditionFailReasons if set.
  if (preconditionFailReasons) {
    let preconditionFailReasonsSet = new Set(preconditionFailReasons);
    const missing = setMinus(
        preconditionFailReasonsSet, notRestoredReasonsSet);
    const extra = setMinus(
        notRestoredReasonsSet, preconditionFailReasonsSet);
    // preconditionFailReasons were reported. PRECONDION_FAIL here.
    assert_implements_optional(
        !(missing.size == 0 && extra.size == 0),
        'Precondition fail reasons are reported.');
  }

  matchReasons(expectedNotRestoredReasonsSet, notRestoredReasonsSet);
}

// A helper function that combines the steps of setting window property,
// navigating away and back, and making assertion on whether BFCache is
// supported.
// This function can be used to check if the current page is eligible for
// BFCache.
async function assertBFCacheEligibility(
    remoteContextHelper, shouldRestoreFromBFCache) {
  await prepareForBFCache(remoteContextHelper);
  // Navigate away and back.
  const newRemoteContextHelper = await remoteContextHelper.navigateToNew();
  await newRemoteContextHelper.historyBack();

  if (shouldRestoreFromBFCache) {
    await assertImplementsBFCacheOptional(remoteContextHelper);
  } else {
    await assertNotRestoredFromBFCache(remoteContextHelper);
  }
}
