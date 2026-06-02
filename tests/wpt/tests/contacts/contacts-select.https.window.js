// META: script=/resources/test-only-api.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/helpers.js
'use strict';

// Verifies that |func|, when invoked, throws a TypeError exception.
async function expectTypeError(func) {
  try {
    await func();
  } catch (e) {
    assert_equals(e.name, 'TypeError');
    return;
  }

  assert_unreached('expected a TypeError, but none was thrown');
}

promise_test(async () => {
  try {
    await navigator.contacts.select(['name']);
    assert_unreached('expected a SecurityError, but none was thrown');
  } catch (e) {
    assert_equals(e.name, 'SecurityError');
  }
}, 'The Contact API requires a user gesture')

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  // At least one property must be provided.
  await expectTypeError(() => navigator.contacts.select());
  await expectTypeError(() => navigator.contacts.select([]));

  // Per WebIDL parsing, no invalid values may be provided.
  await expectTypeError(() =>
      navigator.contacts.select(['']));
  await expectTypeError(() =>
      navigator.contacts.select(['foo']));
  await expectTypeError(() =>
      navigator.contacts.select(['name', 'photo']));

}, 'The Contact API requires valid properties to be provided');

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  // Returns a NULL result, indicating that no results are available.
  setSelectedContacts(null);

  await expectTypeError(() => navigator.contacts.select(['name']));

}, 'The Contact API can fail when the selector cannot be opened');

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  setSelectedContacts([]);

  const properties = await navigator.contacts.getProperties();
  assert_true(properties.length > 0);

  // Requesting the available properties should not fail.
  await navigator.contacts.select(properties);

}, 'Supported contact properties are exposed.');

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  const dwightAddress = {
    country: 'US',
    city: 'Scranton',
    addressLine: ['Schrute Farms'],
  };
  const michaelIcons = [new Blob('image binary data'.split(''), {type: 'image/test'})];

  // Returns two contacts with all information available.
  setSelectedContacts([
      { name: ['Dwight Schrute'], email: ['dwight@schrutefarmsbnb.com'], tel: ['000-0000'], address: [dwightAddress] },
      { name: ['Michael Scott', 'Prison Mike'], email: ['michael@dundermifflin.com'], icon: michaelIcons },
  ]);

  let results = await navigator.contacts.select(['name', 'email', 'icon', 'tel', 'address'], { multiple: true });
  assert_equals(results.length, 2);
  results = results.sort((c1, c2) => JSON.stringify(c1) < JSON.stringify(c2) ? -1 : 1);

  {
    const michael = results[0];

    assert_own_property(michael, 'name');
    assert_own_property(michael, 'email');
    assert_own_property(michael, 'tel');
    assert_own_property(michael, 'address');
    assert_own_property(michael, 'icon');

    assert_array_equals(michael.name, ['Michael Scott', 'Prison Mike']);
    assert_array_equals(michael.email, ['michael@dundermifflin.com']);
    assert_array_equals(michael.tel, []);
    assert_array_equals(michael.address, []);

    assert_equals(michael.icon.length, michaelIcons.length);
    assert_equals(michael.icon[0].type, michaelIcons[0].type);
    assert_equals(michael.icon[0].size, michaelIcons[0].size);
    assert_equals(await michael.icon[0].text(), await michaelIcons[0].text());
  }

  {
    const dwight = results[1];
    assert_own_property(dwight, 'name');
    assert_own_property(dwight, 'email');
    assert_own_property(dwight, 'tel');
    assert_own_property(dwight, 'address');
    assert_own_property(dwight, 'icon');

    assert_array_equals(dwight.name, ['Dwight Schrute']);
    assert_array_equals(dwight.email, ['dwight@schrutefarmsbnb.com']);
    assert_array_equals(dwight.tel, ['000-0000']);
    assert_array_equals(dwight.icon, []);

    assert_equals(dwight.address.length, 1);
    const selectedAddress = dwight.address[0];
    assert_object_equals({
      country: selectedAddress.country,
      city: selectedAddress.city,
      addressLine: selectedAddress.addressLine,
    }, dwightAddress);
  }
}, 'The Contact API correctly returns ContactInfo entries');

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  // Returns two contacts with all information available.
  setSelectedContacts([
      { name: ['Dwight Schrute'], email: ['dwight@schrutefarmsbnb.com'], tel: ['000-0000'] },
      { name: ['Michael Scott', 'Prison Mike'], email: ['michael@dundermifflin.com'] },
  ]);

  const results = await navigator.contacts.select(['name', 'email', 'tel']);
  assert_equals(results.length, 1);

}, 'Only one contact is returned if `multiple` is not set.');

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  // Returns partial information since no e-mail addresses are requested.
  setSelectedContacts([{ name: ['Creed'], email: ['creedthoughts@www.creedthoughts.gov.www'] }]);

  const results = await navigator.contacts.select(['name']);

  assert_equals(results.length, 1);

  {
    const creed = results[0];

    assert_array_equals(creed.name, ['Creed']);
    assert_equals(creed.email, undefined);
    assert_equals(creed.tel, undefined);
  }
}, 'The Contact API does not include fields that were not requested');

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  // Returns partial information since no e-mail addresses are requested.
  setSelectedContacts([{ name: ['Kelly'] }]);

  // First request should work.
  const promise1 = new Promise((resolve, reject) => {
    navigator.contacts.select(['name']).then(resolve)
                                       .catch(e => reject(e.message));
  });

  // Second request should fail (since the first one didn't complete yet).
  const promise2 = new Promise((resolve, reject) => {
    navigator.contacts.select(['name']).then(contacts => reject('This was supposed to fail'))
                                       .catch(e => resolve(e.name));
  });

  const results = await Promise.all([promise1, promise2]);
  const contacts = results[0];
  assert_equals(contacts.length, 1);
  const contact = contacts[0];
  assert_equals(contact.name[0], 'Kelly');
  assert_equals(results[1], 'InvalidStateError');

}, 'The Contact API cannot be used again until the first operation is complete.');

contactsTestWithUserActivation(async (test, setSelectedContacts) => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  iframe.src = 'resources/non-main-frame-select.html';
  await new Promise(resolve => window.addEventListener('message', event => resolve(event.data)))
      .then(data => assert_equals(data.errorMsg, 'InvalidStateError'))
      .finally(() => iframe.remove())

}, 'Test contacts.select() throws an InvalidStateError in a sub-frame');
