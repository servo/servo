// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: Verify that undefined options are handled correctly.
features: [BigInt, Temporal]
---*/

const datetimeEarlier = new Temporal.PlainDateTime(2000, 10, 29, 1, 34, 56, 987, 654, 321);
const datetimeLater = new Temporal.PlainDateTime(2000, 4, 2, 2, 34, 56, 987, 654, 321);
const timeZone = "America/Vancouver";

[
  [datetimeEarlier, 972808496987654321n],
  [datetimeLater, 954671696987654321n],
].forEach(([datetime, expected]) => {
  const explicit = datetime.toZonedDateTime(timeZone, undefined);
  assert.sameValue(explicit.epochNanoseconds, expected, "default disambiguation is compatible");

  const implicit = datetime.toZonedDateTime(timeZone);
  assert.sameValue(implicit.epochNanoseconds, expected, "default disambiguation is compatible");
});
