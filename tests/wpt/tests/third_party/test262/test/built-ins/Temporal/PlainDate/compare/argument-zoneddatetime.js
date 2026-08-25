// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: ZonedDateTime is supported.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");
assert.sameValue(
  Temporal.PlainDate.compare(zdt, new Temporal.PlainDate(1970, 1, 1)),
  0, "same date, ZDT first");
assert.sameValue(
  Temporal.PlainDate.compare(new Temporal.PlainDate(1970, 1, 1), zdt),
  0, "same date, ZDT second");
assert.sameValue(
  Temporal.PlainDate.compare(zdt, new Temporal.PlainDate(1976, 11, 18)),
  -1, "different date, ZDT first");
assert.sameValue(
  Temporal.PlainDate.compare(new Temporal.PlainDate(1976, 11, 18), zdt),
  1, "different date, ZDT second");
