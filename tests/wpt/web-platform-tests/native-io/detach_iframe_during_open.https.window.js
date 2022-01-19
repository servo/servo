// META: title=NativeIO API: open() does not crash in detached iframes.
// META: global=window

promise_test(async testCase => {
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);

  const iframeStorageFoundation = iframe.contentWindow.storageFoundation;

  const openPromise = iframeStorageFoundation.open('test_file');
  iframe.remove();

  // Call getAll() in the main frame. This should keep the test running long
  // enough to catch any crash from the open() call in the removed iframe.
  await storageFoundation.getAll();
}, 'Detaching iframe while storageFoundation.open() settles');
