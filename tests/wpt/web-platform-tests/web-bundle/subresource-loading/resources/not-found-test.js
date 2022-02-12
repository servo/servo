// This test uses a non-existing WebBundle,
//   /web-bundle/resources/wbn/cors/non-existing.wbn.
// The intent of this test is to check if failing to fetch a WebBundle due to
// not found error also makes subresource fetch requests fail.
promise_test(async () => {
  const prefix = '/web-bundle/resources/wbn/';
  const resources = [
    prefix + 'resource.js',
  ];
  const element = await createWebBundleElement(
      prefix + 'non-existing.wbn',
      resources);
  document.body.appendChild(element);

  // Can not fetch a subresource because Web Bundle fetch failed.
  await fetchAndWaitForReject(prefix + 'resource.js');
}, 'Subresource fetch requests for non-existing Web Bundle should fail.');
