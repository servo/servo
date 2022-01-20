promise_test(async () => {
  const response = await fetch('/web-bundle/resources/wbn/nested-sub.wbn');
  assert_true(response.ok);
}, 'A nested bundle can be fetched');

promise_test(async () => {
  await addWebBundleElementAndWaitForError(
      '/web-bundle/resources/wbn/nested-sub.wbn',
      ['/web-bundle/resources/wbn/root.js']);
  const response = await fetch('/web-bundle/resources/wbn/root.js');
  assert_false(response.ok);
}, 'Subresources in a nested bundle should not be loaded');
