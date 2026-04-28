// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: >
  ISO strings cannot have sub-minute UTC offset as a time zone identifier
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1588371240_000_000_000n, "+01:46");

const str = "2021-08-19T17:30:45.123456789-12:12:59.9[-12:12:59.9]";

assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.compare(str, instance),
  `${str} is not a valid ISO string (first argument)`
);
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.compare(instance, str),
  `${str} is not a valid ISO string (second argument)`
);
