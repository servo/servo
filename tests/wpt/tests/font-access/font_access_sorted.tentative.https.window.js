// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/font-asserts.js
// META: script=resources/font-test-utils.js
// META: timeout=long

'use strict';

font_access_test(async t => {
  // The following tests that fonts are sorted. Postscript names are expected to
  // be encoded in a subset of the ASCII character set.
  // See: https://docs.microsoft.com/en-us/typography/opentype/spec/name
  // Should the Postscript name contain characters that are multi-byte, this
  // test may erroneously fail.
  const fonts = await self.queryLocalFonts();
  const fontNames = fonts.map(fontData => fontData.postscriptName);
  const expectedFontNames = [...fontNames].sort();
  assert_array_equals(fontNames, expectedFontNames);
}, 'queryLocalFonts(): fonts are sorted');
