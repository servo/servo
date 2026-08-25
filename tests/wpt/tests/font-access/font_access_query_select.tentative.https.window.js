// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/font-asserts.js
// META: script=resources/font-data.js
// META: script=resources/font-test-utils.js
// META: timeout=long

font_access_test(async t => {
  const testData = getTestData();
  assert_greater_than_equal(
      testData.size, 1, 'Need a least one test font data.');
  const testFont = testData.values().next().value;

  const queryInput = {postscriptNames: [testFont.postscriptName]};
  const fonts = await self.queryLocalFonts(queryInput);

  assert_equals(
      fonts.length, 1, 'The result length should match the test length.');
  assert_font_equals(fonts[0], testFont);
}, 'queryLocalFonts(): valid postscript name in QueryOptions');

font_access_test(async t => {
  const queryInput = {postscriptNames: ['invalid_postscript_name']};
  const fonts = await self.queryLocalFonts(queryInput);

  assert_equals(
      fonts.length, 0,
      'Fonts should not be selected for an invalid postscript name.');
}, 'queryLocalFonts(): invalid postscript name in QueryOptions');

font_access_test(async t => {
  const fonts = await self.queryLocalFonts({});

  assert_greater_than_equal(
      fonts.length, 1,
      'All available fonts should be returned when an empty object is passed.');
}, 'queryLocalFonts(): empty object for QueryOptions.postscriptNames');

font_access_test(async t => {
  const queryInput = {invalidFieldName: []};
  const fonts = await self.queryLocalFonts(queryInput);

  assert_greater_than_equal(
      fonts.length, 1,
      'All available fonts should be returned when an invalid field name for ' +
          'QueryOptions is passed.');
}, 'queryLocalFonts(): invalid QueryOptions field');

font_access_test(async t => {
  const queryInput = {postscriptNames: []};
  const fonts = await self.queryLocalFonts(queryInput);

  assert_equals(
      fonts.length, 0,
      'Fonts should not be selected when an empty list for ' +
          'QueryOptions.postscriptNames is passed.');
}, 'queryLocalFonts(): empty QueryOptions.postscriptNames list');

font_access_test(async t => {
  const fonts = await self.queryLocalFonts(undefined);

  assert_greater_than_equal(
      fonts.length, 1,
      'All available fonts should be returned when undefined is passed for ' +
          'input.');
}, 'queryLocalFonts(): undefined QueryOptions');

const non_ascii_input = [
  {postscriptNames: ['¥']},
  {postscriptNames: ['ß']},
  {postscriptNames: ['🎵']},
  // UTF-16LE, encodes to the same first four bytes as "Ahem" in ASCII.
  {postscriptNames: ['\u6841\u6d65']},
  // U+6C34 CJK UNIFIED IDEOGRAPH (water)
  {postscriptNames: ['\u6C34']},
  // U+1D11E MUSICAL SYMBOL G-CLEF (UTF-16 surrogate pair)
  {postscriptNames: ['\uD834\uDD1E']},
  // U+FFFD REPLACEMENT CHARACTER
  {postscriptNames: ['\uFFFD']},
  // UTF-16 surrogate lead
  {postscriptNames: ['\uD800']},
  // UTF-16 surrogate trail
  {postscriptNames: ['\uDC00']},
];

for (const test of non_ascii_input) {
  font_access_test(async t => {
    const fonts = await self.queryLocalFonts(test);
    assert_equals(
        fonts.length, 0,
        'Fonts should not be selected for non-ASCII character input: ' +
            JSON.stringify(fonts));
  }, `queryLocalFonts(): non-ASCII character input: ${JSON.stringify(test)}`);
}