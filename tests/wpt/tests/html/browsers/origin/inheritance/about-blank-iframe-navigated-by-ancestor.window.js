// META: variant=?same-origin&1-4
// META: variant=?same-origin&5-last
// META: variant=?same-site&1-4
// META: variant=?same-site&5-last
// META: variant=?cross-site&1-4
// META: variant=?cross-site&5-last
// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/subset-tests.js
// META: script=resources/about-blank-navigation.js

// A popup navigates its grandchild to about:blank. The grandchild's parent has
// the variant's origin. The popup is the navigation's initiator, so the
// about:blank document gets the popup's origin, CSP, and base URL, not those
// of its parent.

async function setUp(t, { initiatorHeaders = "", parentHeaders = "", sandbox = null } = {}) {
  const initiator = await openPopupContext(t, hosts.ORIGIN, initiatorHeaders);
  const parent = await addIframeContext(initiator, variantOrigin, parentHeaders);
  await addIframe(parent, `${variantOrigin}/common/blank.html`, sandbox);
  return { initiator, parent };
}

async function navigateToAboutBlank({ initiator, parent }) {
  const loaded = await expectIframeLoad(parent);
  await initiator.execute_script(() => {
    frames[0][0].location.href = "about:blank";
  });
  await loaded();
}

async function assert_initiator_origin({ initiator, parent }) {
  assert_can_script(await inspect(initiator, ["frames", 0, 0]), hosts.ORIGIN, "The initiator");
  const fromParent = await inspect(parent, ["frames", 0]);
  if (variantOrigin == hosts.ORIGIN)
    assert_can_script(fromParent, hosts.ORIGIN, "The same-origin parent");
  else
    assert_cannot_script(fromParent, "The parent");
}

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(contexts);
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(contexts);
  await setDocumentDomain(contexts.initiator);
  assert_can_script(await inspect(contexts.initiator, ["frames", 0, 0]), hosts.ORIGIN, "The initiator, after setting document.domain,");
  assert_cannot_script(await inspect(contexts.parent, ["frames", 0]), "The parent");
}, "about:blank shares its origin with the initiator, so document.domain affects both");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t, { initiatorHeaders: imgSrc(initiatorImageOrigin), parentHeaders: imgSrc(otherImageOrigin) });
  await navigateToAboutBlank(contexts);
  assert_equals(await loadImages(contexts.initiator, ["frames", 0, 0], imageURLs()), "loaded blocked");
}, "about:blank gets the initiator's CSP");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await setBaseURL(contexts.initiator, "/initiator/");
  await setBaseURL(contexts.parent, "/parent/");
  await navigateToAboutBlank(contexts);
  const result = await inspect(contexts.initiator, ["frames", 0, 0]);
  assert_equals(result.error, undefined, "The initiator can script the document");
  assert_equals(result.baseURI, `${hosts.ORIGIN}/initiator/`);
}, "about:blank gets the initiator's base URL");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(contexts);
  let loaded = await expectIframeLoad(contexts.parent);
  await contexts.parent.execute_script(url => {
    frames[0].location.href = url;
  }, [`${variantOrigin}/common/blank.html?next`]);
  await loaded();
  loaded = await expectIframeLoad(contexts.parent);
  await contexts.initiator.execute_script(() => history.back());
  await loaded();
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin again after traversal");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(contexts);
  const loaded = await expectIframeLoad(contexts.parent);
  const error = await contexts.initiator.execute_script(() => {
    try {
      frames[0][0].location.reload();
    } catch (e) {
      return e.name;
    }
  });
  assert_equals(error, undefined, "The initiator can reload the document");
  await loaded();
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin again after reload");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t, { sandbox: "" });
  await navigateToAboutBlank(contexts);
  assert_cannot_script(await inspect(contexts.initiator, ["frames", 0, 0]), "The initiator");
  assert_cannot_script(await inspect(contexts.parent, ["frames", 0]), "The parent");
}, "about:blank in a sandboxed iframe gets an opaque origin");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t, { sandbox: "allow-same-origin" });
  await navigateToAboutBlank(contexts);
  await assert_initiator_origin(contexts);
}, "about:blank in an iframe sandboxed with allow-same-origin gets the initiator's origin");
