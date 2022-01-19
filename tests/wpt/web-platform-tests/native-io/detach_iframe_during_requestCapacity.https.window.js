// META: title=NativeIO API: requestCapacity() does not crash in detached iframes.
// META: global=window

promise_test(async testCase => {
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);

  const iframeStorageFoundation = iframe.contentWindow.storageFoundation;

  const requestPromise = iframeStorageFoundation.requestCapacity(1024 * 1024);
  iframe.remove();

  // Call getAll() in the main frame. This should keep the test running long
  // enough to catch any crash from the requestCapacity() call in the removed
  // iframe.
  await storageFoundation.getAll();
}, 'Detaching iframe while storageFoundation.requestCapacity() settles');
