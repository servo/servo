// META: script=../../html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js

async function assertNotRestoredReasonsEquals(
    remoteContextHelper, url, src, id, name, reasons, children) {
  let result = await remoteContextHelper.executeScript(() => {
    return performance.getEntriesByType('navigation')[0].notRestoredReasons;
  });
  assertReasonsStructEquals(
      result, url, src, id, name, reasons, children);
}

function assertReasonsStructEquals(
    result, url, src, id, name, reasons, children) {
  assert_equals(result.url, url);
  assert_equals(result.src, src);
  assert_equals(result.id, id);
  assert_equals(result.name, name);

  // Reasons should match.
  let expected = new Set(reasons);
  let actual = new Set(result.reasons);
  matchReasons(extractReason(expected), extractReason(actual));

  // Children should match.
  if (children == null) {
    assert_equals(result.children, children);
  } else {
    for (let j = 0; j < children.length; j++) {
      assertReasonsStructEquals(
          result.children[j], children[j].url,
          children[j].src, children[j].id, children[j].name, children[j].reasons,
          children[j].children);
    }
  }
}

function ReasonsInclude(reasons, targetReason) {
  for (const reason of reasons) {
    if (reason.reason == targetReason) {
      return true;
    }
  }
  return false;
}

const BFCACHE_BLOCKING_REASON = "rtc";

async function useBFCacheBlockingFeature(remoteContextHelper) {
  let return_value = await remoteContextHelper.executeScript(() => {
    return new Promise((resolve) => {
      const webRTCInNotRestoredReasonsTests = new RTCPeerConnection();
      webRTCInNotRestoredReasonsTests
          .addIceCandidate({candidate: 'test', sdpMLineIndex: 0})
          .finally(() => {
            resolve(42);
          });
    });
  });
  assert_equals(return_value, 42);
}