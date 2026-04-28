// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Specifically disallowed units for the unit option
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.Duration(0, 0, 0, 4, 5, 6, 7, 987, 654, 321);
const invalidUnits = [
  "era",
  "eras",
];
invalidUnits.forEach((unit) => {
  assert.throws(
    RangeError,
    () => instance.total({ unit }),
    `{ unit: "${unit}" } should not be allowed as an argument to total`
  );
  assert.throws(
    RangeError,
    () => instance.total(unit),
    `"${unit}" should not be allowed as an argument to total`
  );
});
