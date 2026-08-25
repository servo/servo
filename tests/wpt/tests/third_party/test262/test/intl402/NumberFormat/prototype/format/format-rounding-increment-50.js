// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: When set to `50`, roundingIncrement is correctly applied
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
  {roundingIncrement: 50, maximumFractionDigits: 2, minimumFractionDigits: 2},
  {
    '1.500': '1.50',
    '1.625': '1.50',
    '1.750': '2.00',
    '1.875': '2.00',
    '2.000': '2.00',
  }
);

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 50, maximumFractionDigits: 3, minimumFractionDigits: 3},
  {
    '1.0500': '1.050',
    '1.0625': '1.050',
    '1.0750': '1.100',
    '1.0875': '1.100',
    '1.1000': '1.100',
  }
);
