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

// An iframe opens a popup with the variant's origin, which opens a popup of its
// own with that origin. The second popup navigates its opener to about:blank.
// It's the navigation's initiator, so the about:blank document gets its origin,
// CSP, and base URL, not those of the first popup's opener. (Some browsers only
// let a document navigate a cross-origin popup it is related to through its
// opener or ancestors, so the first popup starts out same origin with the
// initiator.)

function closeContextWindow(context) {
  return context.send({ receiver: token(), fn: "() => window.close()", args: [] });
}

// The opener calls the first popup `w`.
async function setUp(t, { openerHeaders = "", initiatorHeaders = "" } = {}) {
  const opener = await createIframeContext(t, hosts.ORIGIN, openerHeaders, iframe => iframe.contentWindow.w?.close());
  const popup = new RemoteContext(token());
  await opener.execute_script(url => {
    window.w = open(url);
  }, [executorURL(variantOrigin, popup.context_id, openerHeaders)]);
  const initiator = new RemoteContext(token());
  t.add_cleanup(() => closeContextWindow(initiator));
  await popup.execute_script(url => {
    // The opener waits for this iframe to go away.
    document.body.append(document.createElement("iframe"));
    open(url);
  }, [executorURL(variantOrigin, initiator.context_id, initiatorHeaders)]);
  await initiator.execute_script(() => {});
  return { opener, popup, initiator };
}

// Waits until `getLength` resolves with `length`, the number of child navigables
// of the first popup's document, and a little more for it to finish loading.
async function waitForPopupLength(t, getLength, length) {
  await t.step_wait(async () => await getLength() === length, `The popup has ${length} child navigables`, 10000, 50);
  await new Promise(resolve => t.step_timeout(resolve, 100));
}

function popupLength(opener) {
  return () => opener.execute_script(() => w.length);
}

async function navigateToAboutBlank(t, { opener, initiator }) {
  await initiator.execute_script(() => {
    opener.location.href = "about:blank";
  });
  await waitForPopupLength(t, popupLength(opener), 0);
}

async function assert_initiator_origin({ opener, initiator }) {
  assert_can_script(await inspect(initiator, ["opener"]), variantOrigin, "The initiator");
  const fromOpener = await inspect(opener, ["w"]);
  if (variantOrigin == hosts.ORIGIN)
    assert_can_script(fromOpener, hosts.ORIGIN, "The same-origin opener");
  else
    assert_cannot_script(fromOpener, "The opener");
}

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(t, contexts);
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(t, contexts);
  await setDocumentDomain(contexts.initiator);
  assert_can_script(await inspect(contexts.initiator, ["opener"]), variantOrigin, "The initiator, after setting document.domain,");
  assert_cannot_script(await inspect(contexts.opener, ["w"]), "The opener");
}, "about:blank shares its origin with the initiator, so document.domain affects both");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t, { openerHeaders: imgSrc(otherImageOrigin), initiatorHeaders: imgSrc(initiatorImageOrigin) });
  await navigateToAboutBlank(t, contexts);
  assert_equals(await loadImages(contexts.initiator, ["opener"], imageURLs()), "loaded blocked");
}, "about:blank gets the initiator's CSP");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await setBaseURL(contexts.opener, "/opener/");
  await setBaseURL(contexts.popup, "/popup/");
  await setBaseURL(contexts.initiator, "/initiator/");
  await navigateToAboutBlank(t, contexts);
  const result = await inspect(contexts.initiator, ["opener"]);
  assert_equals(result.error, undefined, "The initiator can script the document");
  assert_equals(result.baseURI, `${variantOrigin}/initiator/`);
}, "about:blank gets the initiator's base URL");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(t, contexts);
  await contexts.opener.execute_script(url => {
    w.location.href = url;
  }, [`${hosts.ORIGIN}/html/browsers/origin/inheritance/resources/iframe-with-about-blank-iframe.html`]);
  await waitForPopupLength(t, popupLength(contexts.opener), 2);
  await contexts.opener.execute_script(() => w.history.back());
  await waitForPopupLength(t, popupLength(contexts.opener), 0);
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin again after traversal");

subsetTest(promise_test, async t => {
  const contexts = await setUp(t);
  await navigateToAboutBlank(t, contexts);
  const error = await contexts.initiator.execute_script(() => {
    try {
      // The opener waits for this iframe to go away.
      opener.document.body.append(opener.document.createElement("iframe"));
      opener.location.reload();
    } catch (e) {
      return e.name;
    }
  });
  assert_equals(error, undefined, "The initiator can reload the document");
  await waitForPopupLength(t, popupLength(contexts.opener), 0);
  await assert_initiator_origin(contexts);
}, "about:blank gets the initiator's origin again after reload");

subsetTest(promise_test, async t => {
  // A sandboxed iframe without allow-same-origin opens the first popup, which
  // escapes the sandbox. The iframe can't use the dispatcher, so it evaluates
  // what it gets in messages.
  const iframe = document.createElement("iframe");
  iframe.sandbox = "allow-scripts allow-popups allow-popups-to-escape-sandbox";
  iframe.srcdoc = `<script>
    onmessage = async e => {
      let result;
      try {
        result = await eval(e.data);
      } catch (error) {
        result = error.name;
      }
      parent.postMessage(result, "*");
    };
  <\/script>`;
  document.body.append(iframe);
  await new Promise(resolve => iframe.onload = resolve);
  const run = code => new Promise(resolve => {
    window.addEventListener("message", e => resolve(e.data), { once: true });
    iframe.contentWindow.postMessage(code, "*");
  });
  const popup = new RemoteContext(token());
  const initiator = new RemoteContext(token());
  t.add_cleanup(async () => {
    await run("window.w?.close()");
    iframe.remove();
    await closeContextWindow(initiator);
  });

  await run(`window.w = open("${executorURL(variantOrigin, popup.context_id)}"); "opened"`);
  await popup.execute_script(url => {
    // The sandboxed iframe waits for this iframe to go away.
    document.body.append(document.createElement("iframe"));
    open(url);
  }, [executorURL(variantOrigin, initiator.context_id)]);
  await initiator.execute_script(() => {
    opener.location.href = "about:blank";
  });
  await waitForPopupLength(t, () => run("w.length"), 0);

  assert_equals(await run("try { w.document.URL } catch (e) { e.name }"), "SecurityError", "The sandboxed opener can't script the document");
  assert_can_script(await inspect(initiator, ["opener"]), variantOrigin, "The initiator");
}, "about:blank in a popup that escaped the sandbox gets the initiator's origin, not its opener's opaque one");
