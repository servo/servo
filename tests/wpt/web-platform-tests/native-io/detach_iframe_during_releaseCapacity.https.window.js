// META: title=NativeIO API: releaseCapacity() does not crash in detached iframes.
// META: global=window

promise_test(async testCase => {
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);

  const iframeStorageFoundation = iframe.contentWindow.storageFoundation;

  const grantedCapacity =
      await iframeStorageFoundation.requestCapacity(1024 * 1024);

  const releasePromise =
      iframeStorageFoundation.releaseCapacity(grantedCapacity);
  iframe.remove();

  // Call getAll() in the main frame. This should keep the test running long
  // enough to catch any crash from the releaseCapacity() call in the removed
  // iframe.
  await storageFoundation.getAll();
}, 'Detaching iframe while storageFoundation.releaseCapacityCapacity() settles');
