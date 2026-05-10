// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: Checks handling of the unit style.
includes: [testIntl.js]
features: [Intl.NumberFormat-unified]
---*/

assert.throws(TypeError, () => {
  new Intl.NumberFormat([], {
    style: "unit",
  })
});

for (const unit of ["test", "MILE", "kB"]) {
  // Throws RangeError for invalid unit identifier.
  for (const style of [undefined, "decimal", "unit"]) {
    assert.throws(RangeError, () => {
      new Intl.NumberFormat([], { style, unit })
    }, `{ style: ${style}, unit: ${unit} }`);
  }

  const style = "currency";

  // Throws TypeError because "currency" option is missing.
  assert.throws(TypeError, () => {
    new Intl.NumberFormat([], { style, unit })
  }, `{ style: ${style}, unit: ${unit} }`);

  // Throws RangeError for invalid unit identifier.
  assert.throws(RangeError, () => {
    new Intl.NumberFormat([], { style, unit, currency: "USD" })
  }, `{ style: ${style}, unit: ${unit} }`);
}

const nf = new Intl.NumberFormat([], {
  style: "percent",
});
assert.sameValue(nf.resolvedOptions().style, "percent");
assert.sameValue("unit" in nf.resolvedOptions(), false);
assert.sameValue(nf.resolvedOptions().unit, undefined);

function check(unit) {
  const nf = new Intl.NumberFormat([], {
    style: "unit",
    unit,
  });
  const options = nf.resolvedOptions();
  assert.sameValue(options.style, "unit");
  assert.sameValue(options.unit, unit);
}

const units = allSimpleSanctionedUnits();

for (const simpleUnit of units) {
  check(simpleUnit);
  for (const simpleUnit2 of units) {
    check(simpleUnit + "-per-" + simpleUnit2);
    check(simpleUnit2 + "-per-" + simpleUnit);
  }
}
