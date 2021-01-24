'use strict';

font_access_test(async t => {
  const fonts = await navigator.fonts.query({persistentAccess: true});
  const expectedFonts = await filterEnumeration(
      fonts, getEnumerationTestSet({labelFilter: [TEST_SIZE_CATEGORY.small]}));
  const additionalExpectedTables = getMoreExpectedTables(expectedFonts);

  for (const f of expectedFonts) {
    const data = await f.blob();
    assert_not_equals(data.size, 0, 'Returned Blob size slot is populated.');
    const buf = await data.arrayBuffer();
    assert_not_equals(buf.length, 0, 'Returned ArrayBuffer is not empty.');
    assert_equals(data.type, 'application/octet-stream', 'Returned Blob is of type octet-stream.');

    const parsedData = await parseFontData(data);
    assert_not_equals(parsedData.version, 'UNKNOWN', 'SFNT version is a known type.');

    assert_not_equals(parsedData.tables.size, 0, "Should not have tables of size zero.");
    assert_font_has_tables(f.postscriptName, parsedData.tables, BASE_TABLES);

    if (f.postscriptName in additionalExpectedTables) {
      assert_font_has_tables(f.postscriptName,
                             parsedData.tables,
                             additionalExpectedTables[f.postscriptName]);
    }
  }
}, 'blob(): fonts have expected tables that are not empty');
