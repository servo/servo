// META: script=../resources/helpers.js
// META: title=javascript: URL navigation to a string must create a document whose referrer is the navigation initiator

const originalURL = location.href;

const testCases = [
  ["unsafe-url", location.href],
  ["origin", self.origin + "/"],
  ["no-referrer", ""]
];

for (const [referrerPolicyForStartingWindowCreation, expectedReferrer] of testCases) {
  promise_test(async (t) => {
    const meta = document.createElement("meta");
    meta.name = "referrer";
    meta.content = referrerPolicyForStartingWindowCreation;
    t.add_cleanup(() => meta.remove());
    document.head.append(meta);

    const w = await openWindow("/common/blank.html", t);
    const originalReferrer = w.document.referrer;
    assert_equals(originalReferrer, expectedReferrer,
      "Sanity check: opened window's referrer is set correctly");

    // Mess with the current document's URL so that the initiator URL is different. Then, if that
    // shows up as the javascript: URL document's referrer, we know the navigation initiator's URL is
    // being used as the referrer, which is incorrect.
    history.replaceState(undefined, "", "/incorrect-referrer.html");
    t.add_cleanup(() => history.replaceState(undefined, "", originalURL));

    w.location.href = `javascript:'a string<script>opener.postMessage(document.referrer, "*");</script>'`;

    const referrer = await waitForMessage(w);

    assert_equals(referrer, originalReferrer,
      "javascript: URL-created document's referrer equals the previous document's referrer");
  }, `${referrerPolicyForStartingWindowCreation} referrer policy used to create the starting page`);
}
