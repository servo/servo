// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/font-asserts.js
// META: script=resources/font-data.js
// META: script=resources/font-test-utils.js
// META: timeout=long

'use strict';

font_access_test(async t => {
  const fonts = await self.queryLocalFonts();

  // Fonts we know about. Not all expected fonts are included.
  const testData = getTestData();
  // Reduce down the size of results for testing purposes.
  const filteredFonts = filterFonts(fonts, [...testData.keys()]);

  for (const font of filteredFonts) {
    const data = await font.blob();
    assert_not_equals(data.size, 0, 'Blob has a positive size.');
    assert_equals(
        data.type, 'application/octet-stream',
        'Returned Blob is of type octet-stream.');
    const buffer = await data.arrayBuffer();
    assert_not_equals(buffer.length, 0, 'Returned ArrayBuffer is not empty.');

    const parsedData = await parseFontData(data);
    assert_version_info(parsedData.versionTag);
    assert_not_equals(
        parsedData.tables.size, 0, 'Should not have tables of size zero.');
    assert_font_has_tables(font.postscriptName, parsedData.tables, BASE_TABLES);
  }
}, 'FontData.blob(): blob has expected format and parsable table data.');
