// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Negative zero, as an extended year, fails
esid: sec-temporal.plaindate.compare
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);
const invalidStrings = [
  "-000000-10-31",
  "-000000-10-31T00:45",
  "-000000-10-31T00:45+01:00",
  "-000000-10-31T00:45+00:00[UTC]",
];

invalidStrings.forEach((arg) => {
  assert.throws(RangeError,
    () => Temporal.PlainDate.compare(arg, instance),
    "Minus zero is an invalid extended year (first argument)"
  );

  assert.throws(RangeError,
    () => Temporal.PlainDate.compare(instance, arg),
    "Minus zero is an invalid extended year (second argument)"
  );
});
