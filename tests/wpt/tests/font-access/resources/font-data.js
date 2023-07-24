'use strict';

// The OpenType spec mentions that the follow tables are required for a font to
// function correctly. We'll have all the tables listed except for OS/2, which
// is not present in all fonts on Mac OS.
// https://docs.microsoft.com/en-us/typography/opentype/spec/otff#font-tables
const BASE_TABLES = [
  'cmap',
  'head',
  'hhea',
  'hmtx',
  'maxp',
  'name',
  'post',
];

const MAC_FONTS = new Map([
  ['Monaco', {
    postscriptName: 'Monaco',
    fullName: 'Monaco',
    family: 'Monaco',
    style: 'Regular',
  }],
  ['Menlo-Regular', {
    postscriptName: 'Menlo-Regular',
    fullName: 'Menlo Regular',
    family: 'Menlo',
    style: 'Regular',
  }],
]);

const WIN_FONTS = new Map([
  ['Verdana', {
    postscriptName: 'Verdana',
    fullName: 'Verdana',
    family: 'Verdana',
    style: 'Regular',
  }],
]);

const LINUX_FONTS = new Map([
  ['Ahem', {
    postscriptName: 'Ahem',
    fullName: 'Ahem',
    family: 'Ahem',
    style: 'Regular',
  }],
]);

// Returns a map of known system fonts, mapping a font's postscript name to
// FontData.
function getTestData() {
  let output = undefined;
  if (navigator.platform.indexOf("Win") !== -1) {
    output = WIN_FONTS;
  } else if (navigator.platform.indexOf("Mac") !== -1) {
    output = MAC_FONTS;
  } else if (navigator.platform.indexOf("Linux") !== -1) {
    output = LINUX_FONTS;
  }

  assert_not_equals(
      output, undefined, 'Cannot get test set due to unsupported platform.');

  return output;
}