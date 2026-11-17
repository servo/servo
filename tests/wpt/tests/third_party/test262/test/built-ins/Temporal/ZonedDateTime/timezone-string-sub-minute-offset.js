// Copyright (C) 2026 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: >
  Time zone parsing does not accept sub-minute UTC offset as time zone
  identifier
features: [Temporal]
---*/

for (const timeZone of [
  "-12:12:59.9",
  "-12:12:59",
  "-121259",
  "+15:59:00",
  "-170100",
]) {
  assert.throws(
    RangeError,
    () => new Temporal.ZonedDateTime(0n, timeZone),
    `${timeZone} is not a valid time zone identifier`
  );
}
