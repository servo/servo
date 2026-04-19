// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
  Tests that the constructor for Intl.NumberFormat uses appropriate default
  values for its arguments (locales and options).
---*/

const actual = new Intl.NumberFormat().resolvedOptions();
const expected = new Intl.NumberFormat([], Object.create(null)).resolvedOptions();

assert.sameValue(actual.locale, expected.locale);
assert.sameValue(actual.minimumIntegerDigits, expected.minimumIntegerDigits);
assert.sameValue(actual.minimumFractionDigits, expected.minimumFractionDigits);
assert.sameValue(actual.maximumFractionDigits, expected.maximumFractionDigits);
assert.sameValue(actual.numberingSystem, expected.numberingSystem);
assert.sameValue(actual.style, expected.style);
assert.sameValue(actual.useGrouping, expected.useGrouping);
