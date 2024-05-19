// META: script=/common/get-host-info.sub.js

promise_test(async t => {
  let iframeLoadCounter = 0;
  const div = document.createElement('div');
  const iframe = document.createElement('iframe');
  t.add_cleanup(() => iframe.remove());

  iframe.onload = e => iframeLoadCounter++;
  div.append(iframe);
  document.body.append(div);
  assert_equals(iframeLoadCounter, 1, "iframe loads");

  const innerDocument = iframe.contentDocument;
  assert_true(innerDocument !== null, "about:blank Document is reachable");

  document.body.moveBefore(iframe, null);
  assert_equals(iframe.contentDocument, innerDocument, "Document is preserved");
  assert_equals(iframeLoadCounter, 1, "iframe does not reload");
}, "moveBefore(): about:blank iframe's document is preserved");

const kRemoveNewParent = 'remove new parent';
const kRemoveSelf = 'remove self';
const kRemoveSelfViaReplaceChildren = 'remove self via replaceChildren()';
const kRemoveSelfViaInnerHTML = 'remove self via innerHTML';

promise_test(async t => {
  const div = document.createElement('div');
  const iframe = document.createElement('iframe');
  t.add_cleanup(() => iframe.remove());

  const loadPromise = new Promise(resolve => iframe.onload = resolve);
  iframe.src = '/resources/blank.html';

  div.append(iframe);
  document.body.append(div);
  await loadPromise;
  const innerDocument = iframe.contentDocument;

  document.body.moveBefore(iframe, null);
  assert_equals(iframe.contentDocument, innerDocument, "Document is preserved");
}, "moveBefore(): simple same-origin document is preserved");

// This function runs the same test with a few variations. The meat of the test
// loads a cross-origin iframe which asserts that it does not get reloaded.
// Second, we remove the iframe from the parent document in a few different ways
// to trigger initially crashy paths in Chromium during the implementation of
// this feature.
function runTest(removalType) {
  promise_test(async t => {
    let iframeLoadCounter = 0;
    const oldParent = document.createElement('div');
    const newParent = document.createElement('div');
    const iframe = document.createElement('iframe');
    iframe.onload = e => iframeLoadCounter++;
    switch (removalType) {
      case kRemoveNewParent:
        t.add_cleanup(() => newParent.remove());
        break;
      case kRemoveSelf:
        t.add_cleanup(() => iframe.remove());
        break;
      case kRemoveSelfViaReplaceChildren:
        t.add_cleanup(() => newParent.replaceChildren());
        break;
      case kRemoveSelfViaInnerHTML:
        t.add_cleanup(() => {newParent.innerHTML = '';});
        break;
    }

    const loadMessagePromise = new Promise(resolve => window.onmessage = resolve);
    const crossOriginIframeURL = new URL('resources/moveBefore-iframe.html',
        location.href.replace(self.origin, get_host_info().HTTP_REMOTE_ORIGIN));
    iframe.src = crossOriginIframeURL;

    oldParent.append(iframe);
    document.body.append(oldParent, newParent);
    const loadMessage = await loadMessagePromise;
    assert_equals(loadMessage.data, 'loaded');

    const messagePromise = new Promise(resolve => window.onmessage = resolve);
    newParent.moveBefore(iframe, null);
    iframe.contentWindow.postMessage("after moveBefore", "*");
    const message = await messagePromise;
    // If `moveBefore()` behaved just like `insertBefore()`, and reloaded the
    // document, then `message` would contain `loaded` instead of
    // `ack after moveBefore`.
    assert_equals(message.data, 'ack after moveBefore', 'Iframe did not load reload after moveBefore()');
    assert_equals(iframeLoadCounter, 1, "iframe does not fire a second load event");
  }, `moveBefore(): cross-origin iframe is preserved: ${removalType}`);
}

runTest(kRemoveNewParent);
runTest(kRemoveSelf);
runTest(kRemoveSelfViaReplaceChildren);
runTest(kRemoveSelfViaInnerHTML);

promise_test(async t => {
  const iframe1 = document.createElement('iframe');
  iframe1.name = 'iframe1';
  const iframe2 = document.createElement('iframe');
  iframe2.name = 'iframe2';
  const iframe3 = document.createElement('iframe');
  iframe3.name = 'iframe3';

  document.body.append(iframe1, iframe2, iframe3);

  // Assert that the order of iframes in the DOM matches the order of iframes in
  // `window.frames`.
  let iframes = document.querySelectorAll('iframe');
  assert_equals(iframes[0].name, "iframe1", "iframe1 comes first in DOM");
  assert_equals(iframes[1].name, "iframe2", "iframe2 comes second in DOM");
  assert_equals(iframes[2].name, "iframe3", "iframe3 comes last in DOM");
  assert_equals(window.frames[0].name, "iframe1", "iframe1 comes first in frames");
  assert_equals(window.frames[1].name, "iframe2", "iframe2 comes second in frames");
  assert_equals(window.frames[2].name, "iframe3", "iframe3 comes last in frames");

  // Reverse the order of iframes in the DOM.
  document.body.moveBefore(iframe2, iframe1);
  document.body.moveBefore(iframe3, iframe2);

  // Assert that the order of iframes in the DOM is inverse the order of iframes
  // in `window.frames`.
  iframes = document.querySelectorAll('iframe');
  assert_equals(iframes[0].name, "iframe3", "iframe3 comes first in DOM after moveBefore");
  assert_equals(iframes[1].name, "iframe2", "iframe2 comes second in DOM after moveBefore");
  assert_equals(iframes[2].name, "iframe1", "iframe1 comes last in DOM after moveBefore");
  assert_equals(window.frames[0].name, "iframe1", "iframe1 comes first in frames after moveBefore");
  assert_equals(window.frames[1].name, "iframe2", "iframe2 comes second in frames after moveBefore");
  assert_equals(window.frames[2].name, "iframe3", "iframe3 comes last in frames afterMoveBefore");
}, "window.frames ordering does not change due to moveBefore()");
