// Tests for https://mimesniff.spec.whatwg.org/#sniffing-a-mislabeled-binary-resource
//
// When the supplied MIME type is text/plain (the "check-for-apache-bug" case),
// the spec runs the "rules for distinguishing if a resource is text or binary"
// which can only produce text/plain or application/octet-stream.
// It must never produce a scriptable MIME type.

promise_test(async t => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  let loadFired = false;
  let contentDocumentAccessible = true;

  await new Promise((resolve) => {
    iframe.onload = () => {
      loadFired = true;
      try {
        if (!iframe.contentDocument) {
          contentDocumentAccessible = false;
        }
      } catch (e) {
        contentDocumentAccessible = false;
      }
      resolve();
    };

    iframe.onerror = () => resolve();
    iframe.src = 'resources/test-file.zip';
    t.step_timeout(resolve, 2000);
  });

  const downloadTriggered = !loadFired || !contentDocumentAccessible;
  assert_true(downloadTriggered,
    "Binary data served as text/plain should trigger download");
}, "Navigation: binary data served as text/plain should trigger download");

promise_test(async t => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  await new Promise((resolve) => {
    iframe.onload = () => resolve();
    iframe.onerror = () => resolve();
    iframe.src = 'resources/html-content.html';
    t.step_timeout(resolve, 2000);
  });

  // HTML served as text/plain must NOT be rendered as HTML.
  // The spec only allows text/plain or application/octet-stream as outcomes.
  const doc = iframe.contentDocument;
  assert_not_equals(doc, null, "Should be able to access contentDocument");
  assert_equals(doc.getElementById("sniff-marker"), null,
    "HTML elements should not be parsed when served as text/plain");
}, "Navigation: HTML served as text/plain must not be rendered as HTML");

promise_test(async t => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  await new Promise((resolve) => {
    iframe.onload = () => resolve();
    iframe.onerror = () => resolve();
    iframe.src = 'resources/png-image.png';
    t.step_timeout(resolve, 2000);
  });

  // A PNG served as text/plain must not be sniffed into image/png in a
  // browsing context.  The mislabeled-binary algorithm detects binary bytes
  // and produces application/octet-stream, which triggers a download.
  const doc = iframe.contentDocument;
  const imgs = doc ? doc.getElementsByTagName("img") : [];
  assert_equals(imgs.length, 0,
    "PNG served as text/plain should not be rendered as an image in a navigation");
}, "Navigation: PNG served as text/plain must not be sniffed as image");

promise_test(async t => {
  const resp = await fetch('resources/binary-data.bin');
  const contentType = resp.headers.get('Content-Type');

  assert_equals(contentType, "text/plain");
}, "Fetch: binary data served as text/plain must not be sniffed into a privileged type");
