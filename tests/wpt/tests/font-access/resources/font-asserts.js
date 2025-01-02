'use strict';

function assert_font_equals(actualFont, expectedFont) {
  assert_equals(
      actualFont.postscriptName, expectedFont.postscriptName,
      `${actualFont.postscriptName}: postscriptName should match`);
  assert_equals(
      actualFont.fullName, expectedFont.fullName,
      `${actualFont.postscriptName}: fullName should match`);
  assert_equals(
      actualFont.family, expectedFont.family,
      `${actualFont.postscriptName}: family should match`);
  assert_equals(
      actualFont.style, expectedFont.style,
      `${actualFont.postscriptName}: style should match`);
}

function assert_font_has_tables(fontName, actualTables, expectedTables) {
  for (const expectedTable of expectedTables) {
    assert_equals(
        expectedTable.length, 4, 'Table names are always 4 characters long.');
    assert_true(
        actualTables.has(expectedTable),
        `Font ${fontName} did not have required table ${expectedTable}.`);
    assert_greater_than(
        actualTables.get(expectedTable).size, 0,
        `Font ${fontName} has table ${expectedTable} of size 0.`);
  }
}

function assert_version_info(versionTag) {
  // Spec: https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
  assert_true(versionTag === '\x00\x01\x00\x00' ||
      versionTag === 'true' ||
      versionTag === 'typ1' ||
      versionTag === 'OTTO',
      `Invalid sfnt version tag: ${versionTag}`);
}