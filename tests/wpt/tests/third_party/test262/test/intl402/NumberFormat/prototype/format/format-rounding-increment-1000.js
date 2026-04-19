// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: When set to `1000`, roundingIncrement is correctly applied
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
  {roundingIncrement: 1000, maximumFractionDigits: 4, minimumFractionDigits: 4},
  {
    '1.100': '1.1000',
    '1.125': '1.1000',
    '1.150': '1.2000',
    '1.175': '1.2000',
    '1.200': '1.2000',
  }
);

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 1000, maximumFractionDigits: 5, minimumFractionDigits: 5},
  {
    '1.0100': '1.01000',
    '1.0125': '1.01000',
    '1.0150': '1.02000',
    '1.0175': '1.02000',
    '1.0200': '1.02000',
  }
);
