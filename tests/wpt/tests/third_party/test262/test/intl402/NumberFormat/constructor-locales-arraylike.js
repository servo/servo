// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
  Tests that the Intl.NumberFormat constructor accepts Array-like values for the
  locales argument and treats them well.
---*/

const actual = Intl.NumberFormat({
  length: 1,
  0: 'en-US'
}).resolvedOptions();
const expected = Intl.NumberFormat(['en-US']).resolvedOptions();

assert.sameValue(actual.locale, expected.locale);
assert.sameValue(actual.minimumIntegerDigits, expected.minimumIntegerDigits);
assert.sameValue(actual.minimumFractionDigits, expected.minimumFractionDigits);
assert.sameValue(actual.maximumFractionDigits, expected.maximumFractionDigits);
assert.sameValue(actual.numberingSystem, expected.numberingSystem);
assert.sameValue(actual.style, expected.style);
assert.sameValue(actual.useGrouping, expected.useGrouping);
