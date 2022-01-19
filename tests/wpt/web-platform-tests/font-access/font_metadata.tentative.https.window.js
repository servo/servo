//META: script=/resources/testdriver.js
//META: script=/resources/testdriver-vendor.js
//META: script=resources/test-expectations.js

'use strict';

font_access_test(async t => {
  const fonts = await navigator.fonts.query({persistentAccess: true});
  assert_true(Array.isArray(fonts), 'Result of query() should be an Array');
  assert_greater_than_equal(fonts.length, 1, 'Need a least one font');

  fonts.forEach(font => {
    assert_true(font instanceof FontMetadata,
                'Results should be FontMetadata instances');

    // Verify properties and types. This is partially redundant with an IDL
    // test but more domain-specific tests are be done.
    assert_equals(typeof font.postscriptName, 'string');
    assert_true(
      font.postscriptName.split('').every(c => ' ' <= c && c < '\x7f'),
      `postscriptName should be printable ASCII: "${font.postscriptName}"`
    );

    assert_equals(typeof font.fullName, 'string', 'fullName attribute type');
    assert_equals(typeof font.family, 'string', 'family attribute type');
    assert_equals(typeof font.style, 'string', 'style attribute type');

    assert_equals(typeof font.italic, 'boolean', 'italic attribute type');
    assert_equals(typeof font.weight, 'number', 'weight attribute type');
    assert_between_inclusive(
      font.weight, 1, 1000, `${font.postscriptName}: weight attribute range`);

    assert_equals(typeof font.stretch, 'number');
    assert_between_inclusive(
      font.stretch, 0.5, 2, `${font.postscriptName}: stretch attribute range`);
  });
}, 'FontMetadata property types and value ranges');


font_access_test(async t => {
  // Fonts we know about.
  const testSet = getEnumerationTestSet();

  // Get the system fonts.
  let fonts = await navigator.fonts.query({persistentAccess: true});
  assert_true(Array.isArray(fonts), 'Result of query() should be an Array');

  // Filter to the ones we care about.
  fonts = await filterEnumeration(fonts, testSet);
  assert_greater_than_equal(fonts.length, 1, 'Need a least one font');

  const expectations = new Map();
  for (const expectation of testSet) {
    expectations.set(expectation.postscriptName, expectation);
  }

  const results = [];
  fonts.forEach(font => {
    const expectation = expectations.get(font.postscriptName);
    assert_not_equals(expectation, undefined);

    assert_equals(font.fullName, expectation.fullName,
                  `${font.postscriptName}: fullName should match`);
    assert_equals(font.family, expectation.family,
                  `${font.postscriptName}: family should match`);
    assert_equals(font.style, expectation.style,
                  `${font.postscriptName}: style should match`);

    assert_equals(font.italic, expectation.italic,
                  `${font.postscriptName}: italic should match`);
    assert_equals(font.stretch, expectation.stretch,
                  `${font.postscriptName}: stretch should match`);
    assert_equals(font.weight, expectation.weight,
                  `${font.postscriptName}: weight should match`);
  });
}, 'Expected FontMetadata values for for well-known system fonts');
