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

// An iframe with the variant's origin navigates itself to about:blank. Its
// document is the navigation's initiator, so the about:blank document gets
// that document's origin, CSP, and base URL, not those of its parent. A sibling
// iframe with the variant's origin observes it.

async function setUp(t, { iframeHeaders = "", parentHeaders = "", sandbox = null } = {}) {
  const parent = await openPopupContext(t, hosts.ORIGIN, parentHeaders);
  const pipe = iframeHeaders ? `?pipe=${iframeHeaders.substr(1)}` : "";
  await addIframe(parent, `${variantOrigin}/html/browsers/origin/inheritance/resources/navigate-on-message.html${pipe}`, sandbox);
  const sibling = await addIframeContext(parent, variantOrigin);
  return { parent, sibling };
}

async function navigateToAboutBlank({ parent }) {
  const loaded = await expectIframeLoad(parent);
  await parent.execute_script(() => frames[0].postMessage("about:blank", "*"));
  await loaded();
}

async function assert_initiator_origin({ parent, sibling }) {
  assert_can_script(await inspect(sibling, ["parent", 0]), variantOrigin, "A document with the initiator's origin");
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
  await setDocumentDomain(contexts.parent);
  assert_cannot_script(await inspect(contexts.parent, ["frames", 0]), "The parent, after setting document.domain,");
  assert_can_script(await inspect(contexts.sibling, ["parent", 0]), variantOrigin, "A document with the initiator's origin");
}, "about:blank doesn't share its origin with the parent, so document.domain only affects the parent");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t, { iframeHeaders: imgSrc(initiatorImageOrigin), parentHeaders: imgSrc(otherImageOrigin) });
  await navigateToAboutBlank(contexts);
  assert_equals(await loadImages(contexts.sibling, ["parent", 0], imageURLs()), "loaded blocked");
}, "about:blank gets the initiator's CSP");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await setBaseURL(contexts.parent, "/parent/");
  await navigateToAboutBlank(contexts);
  const result = await inspect(contexts.sibling, ["parent", 0]);
  assert_equals(result.error, undefined, "A document with the initiator's origin can script the document");
  assert_equals(result.baseURI, `${variantOrigin}/initiator/`);
}, "about:blank gets the initiator's base URL");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(contexts);
  let loaded = await expectIframeLoad(contexts.parent);
  await contexts.parent.execute_script(url => {
    frames[0].location.href = url;
  }, [`${variantOrigin}/common/blank.html`]);
  await loaded();
  loaded = await expectIframeLoad(contexts.parent);
  await contexts.parent.execute_script(() => history.back());
  await loaded();
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin again after traversal");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(contexts);
  const loaded = await expectIframeLoad(contexts.parent);
  const error = await contexts.sibling.execute_script(() => {
    try {
      parent[0].location.reload();
    } catch (e) {
      return e.name;
    }
  });
  assert_equals(error, undefined, "A document with the initiator's origin can reload the document");
  await loaded();
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin again after reload");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t, { iframeHeaders: "|header(Content-Security-Policy,sandbox allow-scripts)" });
  await navigateToAboutBlank(contexts);
  assert_cannot_script(await inspect(contexts.parent, ["frames", 0]), "The parent");
  assert_cannot_script(await inspect(contexts.sibling, ["parent", 0]), "A document with the initiator's URL's origin");
}, "about:blank gets an opaque origin when the initiator is sandboxed by its CSP");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t, { sandbox: "allow-scripts" });
  await navigateToAboutBlank(contexts);
  assert_cannot_script(await inspect(contexts.parent, ["frames", 0]), "The parent");
  assert_cannot_script(await inspect(contexts.sibling, ["parent", 0]), "A document with the initiator's URL's origin");
}, "about:blank in a sandboxed iframe gets an opaque origin");
