// This test uses a non-existing WebBundle from a non-existent host, which makes
// Web Bundle fetching fail due to a network error. The intent of is to check if
// failing to fetch a WebBundle also makes subresource fetch requests fail.
promise_test(async () => {
  const prefix = 'https://{{hosts[][nonexistent]}}/';
  const resources = [
    prefix + 'resource.js',
  ];
  const link = await addWebBundleElementAndWaitForError(
      prefix + 'non-existing.wbn',
      resources);

  // Can not fetch a subresource because Web Bundle fetch failed.
  await fetchAndWaitForReject(prefix + 'resource.js');
}, 'Subresource fetch requests for non-existing Web Bundle should fail.');