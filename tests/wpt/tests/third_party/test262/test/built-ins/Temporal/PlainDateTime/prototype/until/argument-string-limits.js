// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: ISO strings at the edges of the representable range
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(1976, 11, 18, 15, 23);

const validStrings = [
  "-271821-04-19T00:00:00.000000001",
  "-271821-04-20",
  "+275760-09-13",
  "+275760-09-13T23:59:59.999999999",
];

for (const arg of validStrings) {
  instance.until(arg);
}

const invalidStrings = [
  "-271821-04-19",
  "-271821-04-19T00:00",
  "+275760-09-14",
  "+275760-09-14T00:00",
];

for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `"${arg}" is outside the representable range of PlainDateTime`
  );
}
