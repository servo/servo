'use strict';

// These tests rely on the User Agent providing an implementation of
// platform contacts backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest
const loadChromiumResources = async () => {
  if (!window.MojoInterfaceInterceptor) {
    // Do nothing on non-Chromium-based browsers or when the Mojo bindings are
    // not present in the global namespace.
    return;
  }

  const resources = [
    '/gen/layout_test_data/mojo/public/js/mojo_bindings.js',
    '/gen/third_party/blink/public/mojom/contacts/contacts_manager.mojom.js',
    '/resources/chromium/contacts_manager_mock.js',
  ];

  await Promise.all(resources.map(path => {
    const script = document.createElement('script');
    script.src = path;
    script.async = false;
    const promise = new Promise((resolve, reject) => {
      script.onload = resolve;
      script.onerror = reject;
    });
    document.head.appendChild(script);
    return promise;
  }));
};

// User Agents must provide their own implementation of `WebContacts`,
// which must contain the following this interface:
// class WebContactsTest {
//   /** @param {?Array<!ContactInfo>} contacts */
//   setSelectedContacts(contacts);
// }
async function createWebContactsTest() {
  if (typeof WebContactsTest === 'undefined') {
    await loadChromiumResources();
  }
  assert_true(
    typeof WebContactsTest !== 'undefined',
    'Mojo testing interface is not available.'
  );
  return new WebContactsTest();
}

// Creates a Promise test for |func| given the |description|. The |func| will
// be executed with `setSelectedContacts` which will allow tests to mock out
// the result of calling navigator.contacts.select. `setSelectedContacts`
// accepts a nullable Array of ContactInfos.
function contactsTestWithUserActivation(func, description) {
  promise_test(async test => {
    const webContactsTest = await createWebContactsTest();
    await window.test_driver.bless('request contacts');
    return func(test, contacts => webContactsTest.setSelectedContacts(contacts));
  }, description);
}
