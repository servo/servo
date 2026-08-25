// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: When set to `2`, roundingIncrement is correctly applied
features: [Intl.NumberFormat-v3]
includes: [testIntl.js]
---*/

var locales = [
  new Intl.NumberFormat().resolvedOptions().locale, 'ar', 'de', 'th', 'ja'
];
var numberingSystems = ['arab', 'latn', 'thai', 'hanidec'];

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 2, maximumFractionDigits: 1, minimumFractionDigits: 1},
  {
    '1.20': '1.2',
    '1.25': '1.2',
    '1.30': '1.4',
    '1.35': '1.4',
    '1.40': '1.4',
  }
);

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 2, maximumFractionDigits: 2, minimumFractionDigits: 2},
  {
    '1.020': '1.02',
    '1.025': '1.02',
    '1.030': '1.04',
    '1.035': '1.04',
    '1.040': '1.04',
  }
);
