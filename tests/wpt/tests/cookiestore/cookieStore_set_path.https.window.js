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

promise_test(async testCase => {
  const iframe = document.createElement('iframe');
  iframe.style = 'display: none';
  const loaded = new Promise(resolve => {
    iframe.addEventListener('load', resolve, { once: true });
  });
  iframe.src = '/';
  document.body.appendChild(iframe);
  testCase.add_cleanup(() => iframe.remove());
  await loaded;

  assert_equals(iframe.contentWindow.location.pathname, '/',
                'The document URL has a single path segment');

  await iframe.contentWindow.cookieStore.set({ name: 'cookie-name-root', value: 'cookie-value', path: '' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name-root', path: '/' });
  });

  const internalCookie = await test_driver.get_named_cookie('cookie-name-root');
  assert_equals(internalCookie.path, '/');
}, 'CookieListItem - cookieStore.set with empty string path in a document whose default path is "/"');
