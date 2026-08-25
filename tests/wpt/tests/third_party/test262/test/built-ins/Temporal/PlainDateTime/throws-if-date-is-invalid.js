// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: >
  Throws if any date value is outside the valid bounds.
info: |
  Temporal.PlainDateTime ( isoYear, isoMonth, isoDay [ , hour [ , minute
                           [ , second [ , millisecond [ , microsecond
                           [ , nanosecond [ , calendar ] ] ] ] ] ] ] )

  ...
  16. If IsValidISODate(isoYear, isoMonth, isoDay) is false, throw a RangeError exception.
  ...

features: [Temporal]
---*/

var invalidArgs = [
  [1970, 0, 1],
  [1970, 13, 1],
  [1970, 1, 0],
  [1970, 1, 32],
  [1970, 2, 29],
  [1972, 2, 30],
];

for (var args of invalidArgs) {
  assert.throws(
    RangeError,
    () => new Temporal.PlainDateTime(...args),
    `args = ${JSON.stringify(args)}`
  );
}
