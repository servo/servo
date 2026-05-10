// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: When set to `10`, roundingIncrement is correctly applied
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
  {roundingIncrement: 10, maximumFractionDigits: 2, minimumFractionDigits: 2},
  {
    '1.100': '1.10',
    '1.125': '1.10',
    '1.150': '1.20',
    '1.175': '1.20',
    '1.200': '1.20',
  }
);

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 10, maximumFractionDigits: 3, minimumFractionDigits: 3},
  {
    '1.0100': '1.010',
    '1.0125': '1.010',
    '1.0150': '1.020',
    '1.0175': '1.020',
    '1.0200': '1.020',
  }
);
