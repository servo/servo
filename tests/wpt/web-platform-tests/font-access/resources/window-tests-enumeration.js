'use strict';

const standard_fonts_tests = [
  {},
  {select: []},
];

for (const test of standard_fonts_tests) {
  const inputAsString = JSON.stringify(test) ? JSON.stringify(test) : test;

  font_access_test(async t => {
    const fonts =
        await navigator.fonts.query({persistentAccess: true, ...test});

    assert_fonts_exist(fonts, getEnumerationTestSet());
  }, `query(): standard fonts returned for input: ${inputAsString}`);
}

font_access_test(async t => {
  const fonts = await navigator.fonts.query({persistentAccess: true});
  // The following tests that fonts are sorted. Postscript names are expected to
  // be encoded in a subset of the ASCII character set.
  // See: https://docs.microsoft.com/en-us/typography/opentype/spec/name
  // Should the Postscript name contain characters that are multi-byte, this
  // test may erroneously fail.
  let previousFont = null;
  for (const font of fonts) {
    if (previousFont) {
      assert_true(
          previousFont.postscriptName < font.postscriptName,
          `font is not in expected order. expected: ${
              previousFont.postscriptName} < ${font.postscriptName}`);
    }

    previousFont = font;
  }
}, 'query(): fonts are sorted');

font_access_test(async t => {
  const test = {
    persistentAccess: true,
    select: [getEnumerationTestSet()[0].postscriptName]
  };
  const fonts = await navigator.fonts.query(test);

  assert_postscript_name_exists(fonts, test.select);
  assert_equals(
      fonts.length, test.select.length,
      'The result length should match the test length.');
}, 'query(): fonts are selected for input');

const non_ascii_input = [
  {select: ['¥']},
  {select: ['ß']},
  {select: ['🎵']},
  // UTF-16LE, encodes to the same first four bytes as "Ahem" in ASCII.
  {select: ['\u6841\u6d65']},
  // U+6C34 CJK UNIFIED IDEOGRAPH (water)
  {select: ['\u6C34']},
  // U+1D11E MUSICAL SYMBOL G-CLEF (UTF-16 surrogate pair)
  {select: ['\uD834\uDD1E']},
  // U+FFFD REPLACEMENT CHARACTER
  {select: ['\uFFFD']},
  // UTF-16 surrogate lead
  {select: ['\uD800']},
  // UTF-16 surrogate trail
  {select: ['\uDC00']},
];

for (const test of non_ascii_input) {
  font_access_test(async t => {
    const fonts =
        await navigator.fonts.query({persistentAccess: true, ...test});
    assert_equals(
        fonts.length, 0,
        `There should be no results. Instead got: ${JSON.stringify(fonts)}`);
  }, `query(): No match for input: ${JSON.stringify(test)}`);
}
