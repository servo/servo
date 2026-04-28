// Copyright 2019 Igalia, S.L., Google, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of units.
includes: [testIntl.js]
features: [Intl.NumberFormat-unified]
---*/

function check(unit) {
  const s1 = (123).toLocaleString(undefined, { style: "unit", unit: unit });
  const s2 = (123).toLocaleString();
  assert.notSameValue(s1, s2);
}

const units = allSimpleSanctionedUnits();

for (const simpleUnit of units) {
  check(simpleUnit);
  for (const simpleUnit2 of units) {
    check(simpleUnit + "-per-" + simpleUnit2);
    check(simpleUnit2 + "-per-" + simpleUnit);
  }
}
