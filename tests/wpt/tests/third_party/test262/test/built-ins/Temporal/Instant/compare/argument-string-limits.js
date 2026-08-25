// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: ISO strings at the edges of the representable range
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

const validStrings = [
  "-271821-04-20T00:00Z",
  "-271821-04-19T23:00-01:00",
  "-271821-04-19T00:00:00.000000001-23:59:59.999999999",
  "+275760-09-13T00:00Z",
  "+275760-09-13T01:00+01:00",
  "+275760-09-13T23:59:59.999999999+23:59:59.999999999",
];

for (const arg of validStrings) {
  Temporal.Instant.compare(arg, instance);
  Temporal.Instant.compare(instance, arg);
}

const invalidStrings = [
  "-271821-04-19T23:59:59.999999999Z",
  "-271821-04-19T23:00-00:59:59.999999999",
  "-271821-04-19T00:00:00-23:59:59.999999999",
  "+275760-09-13T00:00:00.000000001Z",
  "+275760-09-13T01:00+00:59:59.999999999",
  "+275760-09-14T00:00+23:59:59.999999999",
];

for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(arg, instance),
    `"${arg}" is outside the representable range of Instant (first argument)`
  );
  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(instance, arg),
    `"${arg}" is outside the representable range of Instant (second argument)`
  );
}
