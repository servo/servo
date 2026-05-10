// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: When set to `25`, roundingIncrement is correctly applied
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
  {roundingIncrement: 25, maximumFractionDigits: 2, minimumFractionDigits: 2},
  {
    '1.2500': '1.25',
    '1.3125': '1.25',
    '1.3750': '1.50',
    '1.4375': '1.50',
    '1.5000': '1.50',
  }
);

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 25, maximumFractionDigits: 3, minimumFractionDigits: 3},
  {
    '1.02500': '1.025',
    '1.03125': '1.025',
    '1.03750': '1.050',
    '1.04375': '1.050',
    '1.05000': '1.050',
  }
);
