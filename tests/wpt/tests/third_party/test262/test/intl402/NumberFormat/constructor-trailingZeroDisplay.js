// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializenumberformat
description: Checks handling of the trailingZeroDisplay option to the NumberFormat constructor.
includes: [compareArray.js]
features: [Intl.NumberFormat-v3]
---*/

const values = [
  [undefined, "auto"],
  ["auto", "auto"],
  ["stripIfInteger", "stripIfInteger"],
  [{toString: function() { return "stripIfInteger"; }}, "stripIfInteger"],
];

for (const [value, expected] of values) {
  const callOrder = [];
  const nf = new Intl.NumberFormat([], {
    get roundingIncrement() {
      callOrder.push("roundingIncrement");
      return 1;
    },
    get trailingZeroDisplay() {
      callOrder.push("trailingZeroDisplay");
      return value;
    }
  });
  const resolvedOptions = nf.resolvedOptions();
  assert("trailingZeroDisplay" in resolvedOptions, "has property for value " + value);
  assert.sameValue(resolvedOptions.trailingZeroDisplay, expected);

  assert.compareArray(callOrder, ["roundingIncrement", "trailingZeroDisplay"]);
}
