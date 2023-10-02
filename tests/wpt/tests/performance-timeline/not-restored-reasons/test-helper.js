// META: script=../../html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js

async function assertNotRestoredReasonsEquals(
    remoteContextHelper, blocked, url, src, id, name, reasons, children) {
  let result = await remoteContextHelper.executeScript(() => {
    return performance.getEntriesByType('navigation')[0].notRestoredReasons;
  });
  assertReasonsStructEquals(
      result, blocked, url, src, id, name, reasons, children);
}

function assertReasonsStructEquals(
    result, blocked, url, src, id, name, reasons, children) {
  assert_equals(result.preventedBackForwardCache, blocked);
  assert_equals(result.url, url);
  assert_equals(result.src, src);
  assert_equals(result.id, id);
  assert_equals(result.name, name);
  // Reasons should match.
  matchReasons(new Set(reasons), new Set(result.reasons));

  // Children should match.
  if (children == null) {
    assert_equals(result.children, children);
  } else {
    for (let j = 0; j < children.length; j++) {
      assertReasonsStructEquals(
          result.children[0], children[0].preventedBackForwardCache, children[0].url,
          children[0].src, children[0].id, children[0].name, children[0].reasons,
          children[0].children);
    }
  }
}

// Requires:
// - /websockets/constants.sub.js in the test file and pass the domainPort
// constant here.
async function useWebSocket(remoteContextHelper) {
  let return_value = await remoteContextHelper.executeScript((domain) => {
    return new Promise((resolve) => {
      var webSocketInNotRestoredReasonsTests = new WebSocket(domain + '/echo');
      webSocketInNotRestoredReasonsTests.onopen = () => { resolve(42); };
    });
  }, [SCHEME_DOMAIN_PORT]);
  assert_equals(return_value, 42);
}