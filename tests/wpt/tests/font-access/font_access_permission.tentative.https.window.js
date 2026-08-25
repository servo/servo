// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await test_driver.bless('enable local font queries');
  const fonts = await self.queryLocalFonts();
  assert_equals(
      fonts.length, 0, 'Fonts are not returned with permission not given.');
}, 'queryLocalFonts(): permission not given');

promise_test(async t => {
  await test_driver.set_permission({name: 'local-fonts'}, 'denied');
  await test_driver.bless('enable local font queries');
  const fonts = await self.queryLocalFonts();
  assert_equals(
      fonts.length, 0, 'Fonts are not returned with permission denied.');
}, 'queryLocalFonts(): permission denied');

promise_test(async t => {
  await test_driver.set_permission({name: 'local-fonts'}, 'granted');
  await test_driver.bless('enable local font queries');
  const fonts = await self.queryLocalFonts();
  assert_greater_than_equal(
      fonts.length, 1, 'Fonts are returned with permission granted.');
}, 'queryLocalFonts(): permission granted');
