// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: >
  Throws if any time value is outside the valid bounds.
info: |
  Temporal.PlainDateTime ( isoYear, isoMonth, isoDay [ , hour [ , minute
                           [ , second [ , millisecond [ , microsecond
                           [ , nanosecond [ , calendar ] ] ] ] ] ] ] )

  ...
  16. If IsValidTime(hour, minute, second, millisecond, microsecond, nanosecond)
      is false, throw a RangeError exception.
  ...

features: [Temporal]
---*/

var invalidArgs = [
  [-1],
  [24],
  [0, -1],
  [0, 60],
  [0, 0, -1],
  [0, 0, 60],
  [0, 0, 0, -1],
  [0, 0, 0, 1000],
  [0, 0, 0, 0, -1],
  [0, 0, 0, 0, 1000],
  [0, 0, 0, 0, 0, -1],
  [0, 0, 0, 0, 0, 1000],
];

for (var args of invalidArgs) {
  assert.throws(
    RangeError,
    () => new Temporal.PlainDateTime(1970, 1, 1, ...args),
    `args = ${JSON.stringify(args)}`
  );
}
