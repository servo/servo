// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializenumberformat
description: Checks handling of the roundingIncrement option to the NumberFormat constructor.
includes: [compareArray.js]
features: [Intl.NumberFormat-v3]
---*/

const values = [
  [undefined, 1],
  [1, 1],
  [2, 2],
  [5, 5],
  [10, 10],
  [20, 20],
  [25, 25],
  [50, 50],
  [100, 100],
  [200, 200],
  [250, 250],
  [500, 500],
  [1000, 1000],
  [2000, 2000],
  [2500, 2500],
  [5000, 5000],
  [true, 1],
  ["2", 2],
  [{valueOf: function() { return 5; }}, 5],
];

for (const [value, expected] of values) {
  const callOrder = [];
  const nf = new Intl.NumberFormat([], {
    get notation() {
      callOrder.push("notation");
      return "standard";
    },
    get roundingIncrement() {
      callOrder.push("roundingIncrement");
      return value;
    },
    minimumFractionDigits: 3
  });
  const resolvedOptions = nf.resolvedOptions();
  assert("roundingIncrement" in resolvedOptions, "has property for value " + value);
  assert.sameValue(resolvedOptions.roundingIncrement, expected);

  assert.compareArray(callOrder, ["notation", "roundingIncrement"]);
}
