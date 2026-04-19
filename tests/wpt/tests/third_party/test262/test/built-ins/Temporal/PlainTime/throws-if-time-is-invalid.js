// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: >
  Throws if any value is outside the valid bounds.
info: |
  Temporal.PlainTime ( [ hour [ , minute [ , second [ , millisecond [ , microsecond [ , nanosecond ] ] ] ] ] ] )

  ...
  8. If IsValidTime(hour, minute, second, millisecond, microsecond, nanosecond) is false, throw a RangeError exception.
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
    () => new Temporal.PlainTime(...args),
    `args = ${JSON.stringify(args)}`
  );
}
