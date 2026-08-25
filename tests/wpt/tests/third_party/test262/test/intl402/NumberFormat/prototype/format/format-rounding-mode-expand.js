// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: >
  When roundingMode is "expand", rounding tends away from zero
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
  {useGrouping: false, roundingMode: 'expand', maximumSignificantDigits: 2},
  {
    '1.101': '1.2',
    '1.15': '1.2',
    '1.1999': '1.2',
    '1.25': '1.3',
    '0': '0',
    '-0': '-0',
    '-1.101': '-1.2',
    '-1.15': '-1.2',
    '-1.1999': '-1.2',
    '-1.25': '-1.3'
  }
);
