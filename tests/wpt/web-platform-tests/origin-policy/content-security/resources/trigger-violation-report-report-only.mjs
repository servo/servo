import { waitForOneSecurityPolicyViolationEvent, waitForImgSuccess } from "./helper.mjs";

promise_test(() => {
  const imgURL = (new URL("/common/security-features/subresource/image.py", document.location)).href;

  return Promise.all([
    waitForOneSecurityPolicyViolationEvent(imgURL).then(blockedURI => {
      assert_equals(blockedURI, imgURL);
    }),
    waitForImgSuccess(imgURL)
  ]);
});
