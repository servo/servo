// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: ISO strings at the edges of the representable range
features: [Temporal]
---*/

const validStrings = [
  "-271821-04-19",
  "-271821-04-19T01:00",
  "+275760-09-13",
  "+275760-09-13T23:00",
];

for (const arg of validStrings) {
  Temporal.PlainDate.from(arg);
}

const invalidStrings = [
  "-271821-04-18",
  "-271821-04-18T23:00",
  "+275760-09-14",
  "+275760-09-14T01:00",
];

for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => Temporal.PlainDate.from(arg),
    `"${arg}" is outside the representable range of PlainDate`
  );
}
