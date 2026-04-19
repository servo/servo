// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: When set to `100`, roundingIncrement is correctly applied
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
  {roundingIncrement: 100, maximumFractionDigits: 3, minimumFractionDigits: 3},
  {
    '1.100': '1.100',
    '1.125': '1.100',
    '1.150': '1.200',
    '1.175': '1.200',
    '1.200': '1.200',
  }
);

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 100, maximumFractionDigits: 4, minimumFractionDigits: 4},
  {
    '1.0100': '1.0100',
    '1.0125': '1.0100',
    '1.0150': '1.0200',
    '1.0175': '1.0200',
    '1.0200': '1.0200',
  }
);
