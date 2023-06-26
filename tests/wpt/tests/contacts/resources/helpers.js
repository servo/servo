'use strict';

// These tests rely on the User Agent providing an implementation of
// platform contacts backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest
async function loadChromiumResources() {
  await import('/resources/chromium/contacts_manager_mock.js');
}

// User Agents must provide their own implementation of `WebContacts`,
// which must contain the following this interface:
// class WebContactsTest {
//   /** @param {?Array<!ContactInfo>} contacts */
//   setSelectedContacts(contacts);
// }
async function createWebContactsTest() {
  if (typeof WebContactsTest === 'undefined') {
    if (isChromiumBased) {
      await loadChromiumResources();
    }
  }
  assert_implements(WebContactsTest, 'WebContactsTest is unavailable.');
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
