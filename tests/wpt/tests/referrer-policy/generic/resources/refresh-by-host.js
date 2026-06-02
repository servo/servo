const kOriginTypeDescriptions = {
  true: "same-origin",
  false: "cross-origin",
}

const kRefreshOptionsByDescription = {
  "meta refresh": "resources/refresh-policy.sub.html",
  "header refresh": "resources/refresh-policy.py",
};

const kExpectEmptyString = "the empty string";
const kExpectOrigin = "origin";
const kExpectFullURL = "full url";

function referrerPolicyExpectationValue(aExpected, aFrame) {
  let expectedReferrer;
  switch (aExpected) {
    case kExpectEmptyString:
      expectedReferrer = "";
      break;
    case kExpectOrigin:
      expectedReferrer = new URL(aFrame.src).origin + "/";
      break;
    case kExpectFullURL:
      expectedReferrer = aFrame.src;
      break;
    default:
      throw new Error(`unexpected referrer type ${aExpected}`);
  }
  return expectedReferrer;
}

function refreshWithPoliciesTest(aExpectedURL, aExpectationsByPolicy) {
  const isSameOrigin = location.origin === new URL(aExpectedURL).origin;
  Object.entries(aExpectationsByPolicy).forEach(([policy, expected]) =>
    Object.entries(kRefreshOptionsByDescription).forEach(([description, refreshFrom]) =>
      promise_test(async t => {
        const originalPath = refreshFrom + "?" + new URLSearchParams({url: aExpectedURL, policy});
        let expectedReferrer = location.href;
        let loadCount = 0;
        const { promise: frameLoaded, resolve: resolveFrameLoaded } = Promise.withResolvers();
        const { promise: messageHandled, resolve: resolveMessageHandled } = Promise.withResolvers();

        const frame = document.createElement("iframe");
        try {
          frame.addEventListener("load", t.step_func(() => {
            loadCount++;
            try {
              if (loadCount === 1) {
                assert_equals(frame.contentWindow.location.href, new URL(originalPath, location).href, "original page loads");
                assert_equals(frame.contentDocument.referrer, expectedReferrer, "referrer is parent frame");

                expectedReferrer = referrerPolicyExpectationValue(expected, frame);
              }
            } finally {
              if (loadCount === 1) {
                resolveFrameLoaded();
              }
            }
          }));

          addEventListener("message", t.step_func(msg => {
            const {referrer, documentReferrer, url} = msg.data;
            try {
              assert_equals(url, aExpectedURL, "refresh page has expected URL");
              assert_equals(referrer, expectedReferrer, "referrer header is previous page");
              assert_equals(documentReferrer, expectedReferrer, "document referrer is previous page");
            } finally {
              resolveMessageHandled();
            }
          }));

          frame.src = originalPath;
          document.body.appendChild(frame);

          await frameLoaded;
          await messageHandled;
        } finally {
          frame.remove();
          t.done();
        }
      }, `${kOriginTypeDescriptions[isSameOrigin]} ${description} with referrer policy "${policy}" refreshes with ${expected} as referrer`)))
}
