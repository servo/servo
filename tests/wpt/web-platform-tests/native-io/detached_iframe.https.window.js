// META: title=NativeIO API: Do not crash in detached iframes.
// META: global=window

promise_test(async testCase => {
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);

  const iframeStorageFoundation = iframe.contentWindow.storageFoundation;
  const frameDOMException = iframe.contentWindow.DOMException;
  iframe.remove();

  await promise_rejects_dom(
    testCase, 'InvalidStateError', frameDOMException,
    iframeStorageFoundation.getAll());
  await promise_rejects_dom(
      testCase, 'InvalidStateError', frameDOMException,
      iframeStorageFoundation.open('test_file'));
  await promise_rejects_dom(
      testCase, 'InvalidStateError', frameDOMException,
      iframeStorageFoundation.rename('test_file', 'test'));
  await promise_rejects_dom(
      testCase, 'InvalidStateError', frameDOMException,
      iframeStorageFoundation.delete('test'));
  await promise_rejects_dom(
      testCase, 'InvalidStateError', frameDOMException,
      iframeStorageFoundation.requestCapacity(10));
  await promise_rejects_dom(
      testCase, 'InvalidStateError', frameDOMException,
      iframeStorageFoundation.releaseCapacity(10));
  await promise_rejects_dom(
      testCase, 'InvalidStateError', frameDOMException,
      iframeStorageFoundation.getRemainingCapacity());
}, 'storageFoundation must return an error when called from detached iframes.');
