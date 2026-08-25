// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tolocalestring
description: Verify that undefined options are handled correctly.
features: [BigInt, Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(957270896_987_650_000n, "UTC");
const defaultFormatter = new Intl.DateTimeFormat('en', {
  year: "numeric",
  month: "numeric",
  day: "numeric",
  hour: "numeric",
  minute: "numeric",
  second: "numeric",
  timeZoneName: "short",
  timeZone: "UTC",
});
const expected = defaultFormatter.format(datetime.toInstant());

const actualExplicit = datetime.toLocaleString('en', undefined);
assert.sameValue(actualExplicit, expected, "default locale is determined by Intl.DateTimeFormat");

const actualImplicit = datetime.toLocaleString('en');
assert.sameValue(actualImplicit, expected, "default locale is determined by Intl.DateTimeFormat");
