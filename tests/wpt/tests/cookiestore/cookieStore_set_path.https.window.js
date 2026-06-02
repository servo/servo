// META: title=Cookie Store API: set()'s path option
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory = currentPath.substr(0, currentPath.lastIndexOf('/'));

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value', path: '' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });

  const internalCookie = await test_driver.get_named_cookie('cookie-name');
  assert_equals(internalCookie.path, currentDirectory);
}, 'CookieListItem - cookieStore.set with empty string path defaults to current URL');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  return promise_rejects_js(testCase, TypeError, cookieStore.set({ name: '__host-cookie-name', value: 'cookie-value', path: '' }));
}, 'CookieListItem - cookieStore.set with empty string path defaults to current URL with __host- prefix');
