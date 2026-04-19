// Copyright 2012 Mozilla Corporation. All rights reserved.
// Copyright 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tolocalestring
description: Tests that toLocaleString handles "thisBigIntValue" correctly.
features: [BigInt]
---*/

var invalidValues = [
  undefined,
  null,
  false,
  "5",
  Symbol(),
  5,
  -1234567.89,
  NaN,
  -Infinity,
  {valueOf: function () { return 5; }},
  {valueOf: function () { return 5n; }},
];

for (const value of invalidValues) {
  assert.throws(TypeError, function() {
    BigInt.prototype.toLocaleString.call(value);
  }, `BigInt.prototype.toLocaleString did not throw with this = ${String(value)}.`);
}
